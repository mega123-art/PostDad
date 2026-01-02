use tokio::sync::mpsc;
use reqwest::Client;

pub enum NetworkEvent {
    RunRequest(String), // The URL
    GotResponse(String),
    Error(String),
}

pub async fn handle_network(
    mut receiver: mpsc::Receiver<NetworkEvent>,
    sender: mpsc::Sender<NetworkEvent>,
) {
    let client = Client::new();

    while let Some(event) = receiver.recv().await {
        match event {
            NetworkEvent::RunRequest(url) => {
                let res = client.get(&url).send().await;
                match res {
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_else(|_| "Error reading body".to_string());
                        let _ = sender.send(NetworkEvent::GotResponse(text)).await;
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
