use reqwest::{Client, Method};
use std::collections::HashMap;
use std::str::FromStr;
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
    },
    GotResponse(String, u16, u128, Vec<String>, String),
    Error(String),
    OAuthCode(String),
    OAuthToken(String),
}

pub async fn handle_network(
    mut receiver: mpsc::Receiver<NetworkEvent>,
    sender: mpsc::Sender<NetworkEvent>,
) {
    let client = Client::new();

    while let Some(event) = receiver.recv().await {
        match event {
            NetworkEvent::RunRequest {
                url,
                method,
                headers,
                body,
                form_data,
                auth,
            } => {
                let start = std::time::Instant::now();

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
                        let cookies: Vec<String> = resp
                            .headers()
                            .get_all("set-cookie")
                            .iter()
                            .filter_map(|h| h.to_str().ok().map(|s| s.to_string()))
                            .collect();

                        let text = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| "Error reading body".to_string());
                        let _ = sender
                            .send(NetworkEvent::GotResponse(
                                text,
                                status,
                                duration,
                                cookies,
                                url.clone(),
                            ))
                            .await;
                    }
                    Err(e) => {
                        let _ = sender.send(NetworkEvent::Error(e.to_string())).await;
                    }
                }
            }
            _ => {}
        }
    }
}
