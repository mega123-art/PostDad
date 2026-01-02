use tokio::sync::mpsc;
use reqwest::{Client, Method};
use std::str::FromStr;

pub enum NetworkEvent {
    RunRequest {
        url: String,
        method: String,
        body: Option<String>,
    }, 
    GotResponse(String, u128),
    Error(String),
}

pub async fn handle_network(
    mut receiver: mpsc::Receiver<NetworkEvent>,
    sender: mpsc::Sender<NetworkEvent>,
) {
    let client = Client::new();

    while let Some(event) = receiver.recv().await {
        match event {
            NetworkEvent::RunRequest { url, method, body } => {
                let start = std::time::Instant::now();
                
                let req_method = Method::from_str(&method).unwrap_or(Method::GET);
                let mut req_builder = client.request(req_method, &url);
                
                if let Some(b) = body {
                    req_builder = req_builder.body(b).header("Content-Type", "application/json");
                }

                let res = req_builder.send().await;
                let duration = start.elapsed().as_millis();
                
                match res {
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_else(|_| "Error reading body".to_string());
                        let _ = sender.send(NetworkEvent::GotResponse(text, duration)).await;
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
