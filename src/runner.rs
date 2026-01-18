use crate::collection::{Collection, RequestConfig};
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
    Started { collection_name: String, total: usize },
    RequestStarted { name: String, index: usize },
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

    let total = requests.len();
    let _ = event_tx
        .send(RunnerEvent::Started {
            collection_name: collection.name.clone(),
            total,
        })
        .await;

    let client = reqwest::Client::new();
    let mut run_result = CollectionRunResult::new(&collection.name, total);

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
        for (key, val) in env_vars {
            let placeholder = format!("{{{{{}}}}}", key);
            url = url.replace(&placeholder, val);
        }

        // Build headers
        let headers = config.headers.clone().unwrap_or_default();

        // Build request body
        let body = config.body.clone();

        // Execute the request
        let start = std::time::Instant::now();
        let result = execute_request(&client, &config.method, &url, &headers, body.as_deref()).await;
        let latency = start.elapsed().as_millis();

        let run_result_item = match result {
            Ok((status, _response_body)) => {
                let expected = config.expected_status.unwrap_or(200);
                let passed = status == expected;
                
                RunResult {
                    name: name.to_string(),
                    method: config.method.clone(),
                    url: url.clone(),
                    status: Some(status),
                    latency_ms: Some(latency),
                    expected_status: Some(expected),
                    passed,
                    error: None,
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
    client: &reqwest::Client,
    method: &str,
    url: &str,
    headers: &HashMap<String, String>,
    body: Option<&str>,
) -> Result<(u16, String), String> {
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
            match response.text().await {
                Ok(text) => Ok((status, text)),
                Err(e) => Err(format!("Failed to read response: {}", e)),
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}
