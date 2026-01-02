use tokio::sync::mpsc;
use reqwest::{Client, Method};
use std::str::FromStr;
use std::collections::HashMap;

pub enum NetworkEvent {
    RunRequest {
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    }, 
    GotResponse(String, u16, u128),
    Error(String),
}

pub async fn handle_network(
    mut receiver: mpsc::Receiver<NetworkEvent>,
    sender: mpsc::Sender<NetworkEvent>,
) {
    let client = Client::new();

    while let Some(event) = receiver.recv().await {
        match event {
            NetworkEvent::RunRequest { url, method, headers, body } => {
                let start = std::time::Instant::now();
                
                let req_method = Method::from_str(&method).unwrap_or(Method::GET);
                let mut req_builder = client.request(req_method, &url);
                
                // Add Headers
                for (k, v) in headers {
                    req_builder = req_builder.header(k, v);
                }

                if let Some(b) = body {
                    req_builder = req_builder.body(b);
                    // If content-type not set in headers, maybe default? 
                    // reqwest usually handles it if needed, or user adds it via H key.
                    // Let's rely on user headers, but default to json if body exists and no content-type?
                    // Nah, raw control is better for Postdad.
                }

                let res = req_builder.send().await;
                let duration = start.elapsed().as_millis();
                
                match res {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let text = resp.text().await.unwrap_or_else(|_| "Error reading body".to_string());
                        let _ = sender.send(NetworkEvent::GotResponse(text, status, duration)).await;
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
