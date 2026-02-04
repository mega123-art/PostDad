use reqwest::{Client, Method};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AuthPayload {
    Bearer(String),
    Basic(String, String),
}

pub enum NetworkEvent {
    RunRequest {
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body: Option<String>,
        form_data: Option<Vec<(String, String, bool)>>,
        auth: Option<AuthPayload>,
        timeout_ms: Option<u64>,
        // SSL Configuration
        ssl_verify: bool,
        ssl_ca_cert: Option<Vec<u8>>, // CA cert bytes (pre-loaded)
        #[allow(dead_code)]
        ssl_client_cert: Option<Vec<u8>>, // Client cert bytes
        #[allow(dead_code)]
        ssl_client_key: Option<Vec<u8>>, // Client key bytes
        // Proxy Configuration
        proxy_url: Option<String>,
        proxy_auth: Option<(String, String)>, // (user, pass)
        no_proxy: Option<String>,
    },
    GotResponse(
        Vec<u8>,
        u16,
        u128,
        Vec<String>,
        String,
        HashMap<String, String>,
    ),
    Error(String),
    OAuthCode(String),
    OAuthToken(String),
    IntrospectSchema {
        url: String,
        headers: HashMap<String, String>,
    },
    GotSchema(String),
    RunGrpc {
        url: String,
        service_method: String,
        proto_path: Option<String>,
        payload: String,
        headers: HashMap<String, String>,
        use_plaintext: bool,
    },
    GotGrpcResponse {
        success: bool,
        body: String,
        error: Option<String>,
        latency_ms: u128,
    },
    ListGrpcServices {
        url: String,
        use_plaintext: bool,
    },
    GotGrpcServices(Vec<String>),
    DescribeGrpcService {
        url: String,
        service: String,
        use_plaintext: bool,
    },
    GotGrpcServiceDescription(String),
}

pub async fn handle_network(
    mut receiver: mpsc::Receiver<NetworkEvent>,
    sender: mpsc::Sender<NetworkEvent>,
) {
    while let Some(event) = receiver.recv().await {
        match event {
            NetworkEvent::RunRequest {
                url,
                method,
                headers,
                body,
                form_data,
                auth,
                timeout_ms,
                ssl_verify,
                ssl_ca_cert,
                ssl_client_cert: _,
                ssl_client_key: _,
                proxy_url,
                proxy_auth,
                no_proxy,
            } => {
                let start = std::time::Instant::now();

                // Build client with SSL configuration
                let timeout = timeout_ms
                    .map(Duration::from_millis)
                    .unwrap_or(Duration::from_secs(30));

                let mut client_builder = Client::builder()
                    .timeout(timeout)
                    .user_agent("PostDad/1.0")
                    .danger_accept_invalid_certs(!ssl_verify);

                // Add custom CA certificate if provided
                if let Some(ca_bytes) = ssl_ca_cert
                    && let Ok(cert) = reqwest::Certificate::from_pem(&ca_bytes)
                {
                    client_builder = client_builder.add_root_certificate(cert);
                }

                // Add client certificate for mTLS if both cert and key provided
                // Note: native-tls does not support Identity::from_pem.
                // To support mTLS with native-tls, we would need PKCS#12 (.p12) support.
                // Disabling this temporarily to fix build on Windows without rustls/cmake.
                /*
                if let (Some(cert_bytes), Some(key_bytes)) = (ssl_client_cert, ssl_client_key) {
                    // Combine cert and key into a single PEM
                    let mut identity_pem = cert_bytes;
                    identity_pem.extend_from_slice(b"\n");
                    identity_pem.extend_from_slice(&key_bytes);

                    if let Ok(identity) = reqwest::Identity::from_pem(&identity_pem) {
                        client_builder = client_builder.identity(identity);
                    }
                }
                */

                // Configure proxy if provided
                if let Some(proxy_str) = proxy_url
                    && let Ok(mut proxy) = reqwest::Proxy::all(&proxy_str)
                {
                    // Add proxy authentication if provided
                    if let Some((user, pass)) = proxy_auth {
                        proxy = proxy.basic_auth(&user, &pass);
                    }
                    client_builder = client_builder.proxy(proxy);
                }

                // Note: no_proxy is passed but reqwest automatically respects
                // the NO_PROXY environment variable, so we don't need to handle it explicitly.
                // It's included in the event for potential future use or logging.
                let _ = no_proxy; // Acknowledge the field is intentionally unused here

                let client = client_builder.build().unwrap_or_else(|_| Client::new());

                let req_method = Method::from_str(&method).unwrap_or(Method::GET);
                let mut req_builder = client.request(req_method, &url);

                for (k, v) in headers {
                    req_builder = req_builder.header(k, v);
                }

                if let Some(a) = auth {
                    match a {
                        AuthPayload::Bearer(token) => {
                            req_builder = req_builder.bearer_auth(token);
                        }
                        AuthPayload::Basic(u, p) => {
                            req_builder = req_builder.basic_auth(u, Some(p));
                        }
                    }
                }

                if let Some(fd) = form_data {
                    let mut form = reqwest::multipart::Form::new();
                    for (k, v, is_file) in fd {
                        if is_file {
                            if let Ok(bytes) = tokio::fs::read(&v).await {
                                let filename = std::path::Path::new(&v)
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("file")
                                    .to_string();

                                let part =
                                    reqwest::multipart::Part::bytes(bytes).file_name(filename);
                                form = form.part(k, part);
                            }
                        } else {
                            form = form.text(k, v);
                        }
                    }
                    req_builder = req_builder.multipart(form);
                } else if let Some(b) = body {
                    req_builder = req_builder.body(b);
                }

                let res = req_builder.send().await;
                let duration = start.elapsed().as_millis();

                match res {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let mut resp_headers = HashMap::new();
                        for (k, v) in resp.headers() {
                            resp_headers.insert(
                                k.as_str().to_string(),
                                v.to_str().unwrap_or("").to_string(),
                            );
                        }

                        let cookies: Vec<String> = resp
                            .headers()
                            .get_all("set-cookie")
                            .iter()
                            .filter_map(|h| h.to_str().ok().map(|s| s.to_string()))
                            .collect();

                        let bytes = resp
                            .bytes()
                            .await
                            .map(|b| b.to_vec())
                            .unwrap_or_else(|_| Vec::new());

                        let _ = sender
                            .send(NetworkEvent::GotResponse(
                                bytes,
                                status,
                                duration,
                                cookies,
                                url.clone(),
                                resp_headers,
                            ))
                            .await;
                    }
                    Err(e) => {
                        let _ = sender.send(NetworkEvent::Error(e.to_string())).await;
                    }
                }
            }
            NetworkEvent::IntrospectSchema { url, headers } => {
                let client = Client::builder()
                    .timeout(Duration::from_secs(30))
                    .user_agent("PostDad/1.0")
                    .build()
                    .unwrap_or_else(|_| Client::new());

                let query = r#"{"query": "query Introspection { __schema { types { name fields { name } } } }"}"#;
                let mut req_builder = client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .body(query);

                for (k, v) in headers {
                    req_builder = req_builder.header(k, v);
                }

                if let Ok(resp) = req_builder.send().await
                    && let Ok(text) = resp.text().await
                {
                    let _ = sender.send(NetworkEvent::GotSchema(text)).await;
                }
            }
            NetworkEvent::RunGrpc {
                url,
                service_method,
                proto_path,
                payload,
                headers,
                use_plaintext,
            } => {
                // Execute gRPC request using grpcurl
                let result = crate::net::grpc::execute_grpc_request(
                    &url,
                    &service_method,
                    proto_path.as_deref(),
                    &payload,
                    &headers,
                    use_plaintext,
                );

                let _ = sender
                    .send(NetworkEvent::GotGrpcResponse {
                        success: result.success,
                        body: result.body,
                        error: result.error,
                        latency_ms: result.latency_ms,
                    })
                    .await;
            }
            NetworkEvent::ListGrpcServices { url, use_plaintext } => {
                match crate::net::grpc::list_services(&url, use_plaintext) {
                    Ok(services) => {
                        let _ = sender.send(NetworkEvent::GotGrpcServices(services)).await;
                    }
                    Err(e) => {
                        let _ = sender
                            .send(NetworkEvent::GotGrpcServices(vec![format!("Error: {}", e)]))
                            .await;
                    }
                }
            }
            NetworkEvent::DescribeGrpcService {
                url,
                service,
                use_plaintext,
            } => match crate::net::grpc::describe_service(&url, &service, use_plaintext) {
                Ok(desc) => {
                    let _ = sender
                        .send(NetworkEvent::GotGrpcServiceDescription(desc))
                        .await;
                }
                Err(e) => {
                    let _ = sender
                        .send(NetworkEvent::GotGrpcServiceDescription(format!(
                            "Error: {}",
                            e
                        )))
                        .await;
                }
            },
            _ => {}
        }
    }
}
