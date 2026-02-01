use crate::collection::{Collection, RequestConfig};
use crate::scripting;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Result of running a single request in the collection
#[derive(Clone, Debug)]
pub struct RunResult {
    pub name: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub latency_ms: Option<u128>,
    pub expected_status: Option<u16>,
    pub passed: bool,
    pub error: Option<String>,
    pub tests: Vec<(String, bool)>,
}

/// Overall result of running a collection
#[derive(Clone, Debug, Default)]
pub struct CollectionRunResult {
    pub collection_name: String,
    pub results: Vec<RunResult>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub running: bool,
    pub current_index: usize,
}

impl CollectionRunResult {
    pub fn new(collection_name: &str, total: usize) -> Self {
        Self {
            collection_name: collection_name.to_string(),
            results: Vec::new(),
            total,
            passed: 0,
            failed: 0,
            running: true,
            current_index: 0,
        }
    }

    pub fn add_result(&mut self, result: RunResult) {
        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result);
        self.current_index += 1;
    }

    pub fn finish(&mut self) {
        self.running = false;
    }
}

/// Event sent from the runner to update the UI
#[derive(Clone, Debug)]
pub enum RunnerEvent {
    Started {
        collection_name: String,
        total: usize,
    },
    RequestStarted {
        name: String,
        index: usize,
    },
    RequestCompleted(RunResult),
    Finished(CollectionRunResult),
    Error(String),
}

/// Runs a collection of requests sequentially
pub async fn run_collection(
    collection: &Collection,
    env_vars: &HashMap<String, String>,
    event_tx: mpsc::Sender<RunnerEvent>,
) {
    let requests: Vec<(&String, &RequestConfig)> = {
        let mut items: Vec<_> = collection.requests.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        items
    };

    if requests.is_empty() {
        let _ = event_tx
            .send(RunnerEvent::Error(
                "Collection contains no requests".to_string(),
            ))
            .await;
        return;
    }

    let total = requests.len();
    let _ = event_tx
        .send(RunnerEvent::Started {
            collection_name: collection.name.clone(),
            total,
        })
        .await;

    let mut run_result = CollectionRunResult::new(&collection.name, total);
    let mut current_env_vars = env_vars.clone();

    for (index, (name, config)) in requests.iter().enumerate() {
        // Notify that we're starting this request
        let _ = event_tx
            .send(RunnerEvent::RequestStarted {
                name: name.to_string(),
                index,
            })
            .await;

        // Process URL with environment variables
        let mut url = config.url.clone();
        for (key, val) in &current_env_vars {
            let placeholder = format!("{{{{{}}}}}", key);
            url = url.replace(&placeholder, val);
        }

        // Build headers
        let mut headers = config.headers.clone().unwrap_or_default();

        // Build request body
        let mut body = config.body.clone();

        // Run Pre-Request Script
        if let Some(script) = &config.pre_request_script {
            if !script.trim().is_empty() {
                let script_result = scripting::run_script(
                    script,
                    &config.method,
                    &url,
                    &headers,
                    body.as_deref().unwrap_or(""),
                    &current_env_vars,
                );

                // Apply script results
                headers = script_result.headers;
                if let Some(new_body) = script_result.body_override {
                    body = Some(new_body);
                }
                if let Some(new_url) = script_result.url_override {
                    url = new_url;
                }
                // Merge variables
                for (k, v) in script_result.variables {
                    current_env_vars.insert(k, v);
                }
            }
        }

        // Execute the request
        let start = std::time::Instant::now();
        let result =
            execute_request(&config.method, &url, &headers, body.as_deref(), config.timeout_ms).await;
        let latency = start.elapsed().as_millis();

        let run_result_item = match result {
            Ok((status, response_body, response_headers)) => {
                let expected = config.expected_status.unwrap_or(200);
                let status_passed = status == expected;
                let mut tests = Vec::new();

                // Run Post-Request Script
                if let Some(script) = &config.post_request_script {
                    if !script.trim().is_empty() {
                        let script_res = scripting::run_post_script(
                            script,
                            status,
                            &response_body,
                            &response_headers,
                            latency,
                        );
                        tests = script_res.tests;
                    }
                }

                // Passed if status matches AND all tests passed
                let tests_passed = tests.iter().all(|(_, p)| *p);
                // If expected status is NOT set in config, maybe we shouldn't fail on status?
                // But typically 200 is default.
                // Logic: If tests exist, they override status check? No, usually AND.
                // Postman: Status check is just another test.
                // PostDad: expected_status is a distinct field.
                let passed = status_passed && tests_passed;

                RunResult {
                    name: name.to_string(),
                    method: config.method.clone(),
                    url: url.clone(),
                    status: Some(status),
                    latency_ms: Some(latency),
                    expected_status: Some(expected),
                    passed,
                    error: None,
                    tests,
                }
            }
            Err(e) => RunResult {
                name: name.to_string(),
                method: config.method.clone(),
                url: url.clone(),
                status: None,
                latency_ms: Some(latency),
                expected_status: config.expected_status,
                passed: false,
                error: Some(e),
                tests: Vec::new(),
            },
        };

        let _ = event_tx
            .send(RunnerEvent::RequestCompleted(run_result_item.clone()))
            .await;
        run_result.add_result(run_result_item);
    }

    run_result.finish();
    let _ = event_tx.send(RunnerEvent::Finished(run_result)).await;
}

async fn execute_request(
    method: &str,
    url: &str,
    headers: &HashMap<String, String>,
    body: Option<&str>,
    timeout_ms: Option<u64>,
) -> Result<(u16, String, HashMap<String, String>), String> {
    use std::time::Duration;
    
    // Build client with timeout and default User-Agent
    let client = if let Some(ms) = timeout_ms {
        reqwest::Client::builder()
            .timeout(Duration::from_millis(ms))
            .user_agent("PostDad/1.0")
            .build()
            .map_err(|e| format!("Failed to create client: {}", e))?
    } else {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("PostDad/1.0")
            .build()
            .map_err(|e| format!("Failed to create client: {}", e))?
    };

    let method = match method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let mut request = client.request(method, url);

    for (key, value) in headers {
        request = request.header(key, value);
    }

    if let Some(body_content) = body {
        request = request.body(body_content.to_string());
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let mut headers = HashMap::new();
            for (k, v) in response.headers() {
                headers.insert(k.as_str().to_string(), v.to_str().unwrap_or("").to_string());
            }

            match response.text().await {
                Ok(text) => Ok((status, text, headers)),
                Err(e) => Err(format!("Failed to read response: {}", e)),
            }
        }
        Err(e) => {
            if e.is_timeout() {
                Err(format!("Request timed out after {}ms", timeout_ms.unwrap_or(30000)))
            } else {
                Err(format!("Request failed: {}", e))
            }
        }
    }
}
