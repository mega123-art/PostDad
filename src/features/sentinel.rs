use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct SentinelConfig {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub interval_secs: u64,
    pub failure_keyword: Option<String>,
}

#[derive(Debug)]
pub struct SentinelResult {
    pub latency_ms: u64,
    pub status: Result<u16, String>,
    pub timestamp: u64,
}

pub struct SentinelState {
    pub is_running: bool,
    pub latency_history: VecDeque<u64>,
    pub status_history: VecDeque<u16>,
    pub timestamp_history: VecDeque<u64>,
    pub total_checks: u64,
    pub failed_checks: u64,
    pub last_latency: u64,
    pub last_status: Option<u16>,
    // Channel to stop the background task
    pub stop_tx: Option<mpsc::Sender<()>>,
}

impl SentinelState {
    pub fn new() -> Self {
        Self {
            is_running: false,
            latency_history: VecDeque::with_capacity(100),
            status_history: VecDeque::with_capacity(100),
            timestamp_history: VecDeque::with_capacity(100),
            total_checks: 0,
            failed_checks: 0,
            last_latency: 0,
            last_status: None,
            stop_tx: None,
        }
    }

    pub fn add_result(&mut self, result: SentinelResult) {
        self.total_checks += 1;
        self.last_latency = result.latency_ms;

        if self.latency_history.len() >= 100 {
            self.latency_history.pop_front();
            self.timestamp_history.pop_front();
        }
        self.latency_history.push_back(result.latency_ms);
        self.timestamp_history.push_back(result.timestamp);

        match result.status {
            Ok(code) => {
                self.last_status = Some(code);
                if code >= 400 {
                    // Consider 4xx/5xx as "failed" checks for health purposes?
                    // Or maybe just 5xx? Let's say we mark non-2xx as interesting,
                    // but explicitly track network errors or 5xx as failures.
                    if code >= 500 {
                        self.failed_checks += 1;
                    }
                }

                if self.status_history.len() >= 100 {
                    self.status_history.pop_front();
                }
                self.status_history.push_back(code);
            }
            Err(_) => {
                self.failed_checks += 1;
                // Store 0 for status on network error? Or 503?
                self.last_status = None;
                if self.status_history.len() >= 100 {
                    self.status_history.pop_front();
                }
                self.status_history.push_back(0); // 0 indicates network error
            }
        }
    }
    pub fn save_history(&self) -> Result<String, std::io::Error> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filename = format!("sentinel_log_{}.csv", timestamp);
        let mut content = String::from("index,timestamp,status,latency_ms\n");

        let len = self.latency_history.len();
        for i in 0..len {
            let lat = self.latency_history.get(i).unwrap_or(&0);
            let status = self.status_history.get(i).unwrap_or(&0);
            let ts = self.timestamp_history.get(i).unwrap_or(&0);
            content.push_str(&format!("{},{},{},{}\n", i, ts, status, lat));
        }

        std::fs::write(&filename, content)?;
        Ok(filename)
    }
}

pub async fn run_sentinel_task(
    config: SentinelConfig,
    res_tx: mpsc::Sender<SentinelResult>,
    mut stop_rx: mpsc::Receiver<()>,
) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut interval = tokio::time::interval(Duration::from_secs(config.interval_secs));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                 let start = Instant::now();
                 let method = match config.method.as_str() {
                    "POST" => reqwest::Method::POST,
                    "PUT" => reqwest::Method::PUT,
                    "DELETE" => reqwest::Method::DELETE,
                    "PATCH" => reqwest::Method::PATCH,
                    _ => reqwest::Method::GET,
                };

                let mut req_builder = client.request(method, &config.url);
                for (k, v) in &config.headers {
                    req_builder = req_builder.header(k, v);
                }
                if let Some(body) = &config.body {
                    req_builder = req_builder.body(body.clone());
                }

                let result = req_builder.send().await;
                let latency = start.elapsed().as_millis() as u64;

                let status = match result {
                    Ok(resp) => {
                        let code = resp.status().as_u16();
                        // Check failure keyword if configured
                        if let Some(ref keyword) = config.failure_keyword {
                            // If we can read the body, check for the keyword
                            // Note: reading body consumes it, but we don't need it after this
                            if let Ok(text) = resp.text().await {
                                if text.contains(keyword) {
                                    // Treat as 500
                                    Ok(500)
                                } else {
                                    Ok(code)
                                }
                            } else {
                                Ok(code)
                            }
                        } else {
                            Ok(code)
                        }
                    },
                    Err(e) => Err(e.to_string()),
                };

                let res = SentinelResult {
                    latency_ms: latency,
                    status,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                if res_tx.send(res).await.is_err() {
                    break;
                }
            }
            _ = stop_rx.recv() => {
                break;
            }
        }
    }
}
