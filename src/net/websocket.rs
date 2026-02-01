use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Represents a WebSocket message in the history
#[derive(Clone, Debug)]
pub struct WsMessage {
    pub content: String,
    pub is_sent: bool,
    pub timestamp: std::time::Instant,
}

/// Commands that can be sent to the WebSocket task
pub enum WsCommand {
    Connect(String),
    Send(String),
    Disconnect,
}

/// Events that the WebSocket task sends back to the UI
#[derive(Clone, Debug)]
pub enum WsEvent {
    Connected,
    Disconnected,
    Message(String),
    Error(String),
}

/// Handle to control an active WebSocket connection
pub struct WsHandle {
    pub command_tx: mpsc::Sender<WsCommand>,
}

/// Spawns a WebSocket handler task that manages the connection
pub fn spawn_ws_handler(event_tx: mpsc::Sender<WsEvent>) -> WsHandle {
    let (command_tx, mut command_rx) = mpsc::channel::<WsCommand>(32);

    tokio::spawn(async move {
        let ws_stream: Arc<
            Mutex<
                Option<
                    futures_util::stream::SplitSink<
                        tokio_tungstenite::WebSocketStream<
                            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                        >,
                        Message,
                    >,
                >,
            >,
        > = Arc::new(Mutex::new(None));

        while let Some(cmd) = command_rx.recv().await {
            match cmd {
                WsCommand::Connect(url) => {
                    let event_tx_clone = event_tx.clone();
                    let ws_stream_clone = ws_stream.clone();

                    match connect_async(&url).await {
                        Ok((stream, _)) => {
                            let (write, mut read) = stream.split();
                            {
                                let mut ws = ws_stream_clone.lock().await;
                                *ws = Some(write);
                            }
                            let _ = event_tx_clone.send(WsEvent::Connected).await;

                            // Spawn a task to read incoming messages
                            let event_tx_read = event_tx_clone.clone();
                            let ws_stream_read = ws_stream_clone.clone();
                            tokio::spawn(async move {
                                while let Some(msg_result) = read.next().await {
                                    match msg_result {
                                        Ok(msg) => {
                                            let text = match msg {
                                                Message::Text(t) => t.to_string(),
                                                Message::Binary(b) => {
                                                    format!("[Binary: {} bytes]", b.len())
                                                }
                                                Message::Ping(_) => "[Ping]".to_string(),
                                                Message::Pong(_) => "[Pong]".to_string(),
                                                Message::Close(_) => {
                                                    let _ = event_tx_read
                                                        .send(WsEvent::Disconnected)
                                                        .await;
                                                    let mut ws = ws_stream_read.lock().await;
                                                    *ws = None;
                                                    break;
                                                }
                                                Message::Frame(_) => continue,
                                            };
                                            let _ =
                                                event_tx_read.send(WsEvent::Message(text)).await;
                                        }
                                        Err(e) => {
                                            let _ = event_tx_read
                                                .send(WsEvent::Error(e.to_string()))
                                                .await;
                                            let mut ws = ws_stream_read.lock().await;
                                            *ws = None;
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            let _ = event_tx_clone
                                .send(WsEvent::Error(format!("Connection failed: {}", e)))
                                .await;
                        }
                    }
                }
                WsCommand::Send(msg) => {
                    let ws_stream_clone = ws_stream.clone();
                    let event_tx_clone = event_tx.clone();

                    let mut ws = ws_stream_clone.lock().await;
                    if let Some(ref mut writer) = *ws {
                        if let Err(e) = writer.send(Message::Text(msg.into())).await {
                            let _ = event_tx_clone
                                .send(WsEvent::Error(format!("Send failed: {}", e)))
                                .await;
                        }
                    } else {
                        let _ = event_tx_clone
                            .send(WsEvent::Error("Not connected".to_string()))
                            .await;
                    }
                }
                WsCommand::Disconnect => {
                    let ws_stream_clone = ws_stream.clone();
                    let event_tx_clone = event_tx.clone();

                    let mut ws = ws_stream_clone.lock().await;
                    if let Some(ref mut writer) = *ws {
                        let _ = writer.send(Message::Close(None)).await;
                    }
                    *ws = None;
                    let _ = event_tx_clone.send(WsEvent::Disconnected).await;
                }
            }
        }
    });

    WsHandle { command_tx }
}
