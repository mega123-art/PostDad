use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct StressConfig {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub concurrency: u32,
    pub duration_secs: u64,
}

#[derive(Clone, Debug, Default)]
pub struct StressStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub errors_count: u64,
    pub avg_latency_ms: f64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub p50_latency_ms: u64,
    pub p90_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub rps: f64,
    pub status_dist: HashMap<u16, u64>,
}

#[derive(Debug)]
pub enum StressEvent {
    Progress {
        requests_done: u64,
        elapsed_secs: u64,
    },
    Finished(StressStats),
    Error(String),
}

pub async fn run_stress_test(config: StressConfig, tx: mpsc::Sender<StressEvent>) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(config.concurrency as usize)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let start_time = Instant::now();
    let duration = Duration::from_secs(config.duration_secs);
    let (res_tx, mut res_rx) = mpsc::channel(1000);

    let config = Arc::new(config);
    // Spawn workers
    for _ in 0..config.concurrency {
        let client = client.clone();
        let config = config.clone();
        let res_tx = res_tx.clone();

        tokio::spawn(async move {
            while start_time.elapsed() < duration {
                let req_start = Instant::now();
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
                let latency = req_start.elapsed().as_millis() as u64;

                let status = match result {
                    Ok(resp) => Ok(resp.status().as_u16()),
                    Err(e) => Err(e.to_string()),
                };

                if res_tx.send((latency, status)).await.is_err() {
                    break;
                }
            }
        });
    }

    // Drop our sender so the channel closes when workers are done
    drop(res_tx);

    let mut latencies = Vec::new();
    let mut status_dist = HashMap::new();
    let mut errors_count = 0;
    let mut last_tick = Instant::now();

    while let Some((latency, status)) = res_rx.recv().await {
        latencies.push(latency);
        match status {
            Ok(code) => {
                *status_dist.entry(code).or_insert(0) += 1;
            }
            Err(_) => {
                errors_count += 1;
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(500) {
            let _ = tx
                .send(StressEvent::Progress {
                    requests_done: latencies.len() as u64,
                    elapsed_secs: start_time.elapsed().as_secs(),
                })
                .await;
            last_tick = Instant::now();
        }
    }

    // Calculate stats
    let total = latencies.len() as u64;
    if total > 0 {
        latencies.sort_unstable();
        let sum: u64 = latencies.iter().sum();
        let avg = sum as f64 / total as f64;
        let min = *latencies.first().unwrap();
        let max = *latencies.last().unwrap();
        let p50 = latencies[(total as f64 * 0.5) as usize];
        let p90 = latencies[(total as f64 * 0.9) as usize];
        let p99 = latencies[(total as f64 * 0.99) as usize];
        let duration_actual = start_time.elapsed().as_secs_f64();
        let rps = total as f64 / duration_actual;
        let success = total - errors_count;

        let stats = StressStats {
            total_requests: total,
            successful_requests: success,
            failed_requests: total - success, // Count non-200s? No, failed means network error here.
            errors_count,
            avg_latency_ms: avg,
            min_latency_ms: min,
            max_latency_ms: max,
            p50_latency_ms: p50,
            p90_latency_ms: p90,
            p99_latency_ms: p99,
            rps,
            status_dist,
        };

        let _ = tx.send(StressEvent::Finished(stats)).await;
    } else {
        let _ = tx
            .send(StressEvent::Error("No requests completed".to_string()))
            .await;
    }
}
