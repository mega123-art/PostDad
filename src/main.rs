use crate::app::{App, InputMode};
use crate::network::{handle_network, NetworkEvent};
use std::io;
use tokio::sync::mpsc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use serde_json::Value;

mod app;
mod ui;
mod handler;
mod network;
mod collection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Channels for "Crazy" Async Performance
    let (ui_tx, network_rx) = mpsc::channel(32);
    let (network_tx, mut ui_rx) = mpsc::channel(32);

    // 2. Spawn the Background "Dad" Worker
    tokio::spawn(async move {
        handle_network(network_rx, network_tx).await;
    });

    // 3. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        // 4. Handle Background Messages (Did the API respond?)
        while let Ok(event) = ui_rx.try_recv() {
            match event {
                NetworkEvent::GotResponse(text) => {
                    // Try to parse as JSON for the explorer
                    if let Ok(val) = serde_json::from_str::<Value>(&text) {
                        let root = crate::app::JsonEntry::from_value("root".to_string(), &val, 0);
                        // Wrap in a doc-root if needed or just use a list
                        app.response_json = Some(vec![root]);
                    } else {
                        app.response_json = None;
                    }

                    app.response = Some(text);
                    app.is_loading = false;
                }
                NetworkEvent::Error(e) => {
                    app.response = Some(format!("Error: {}", e));
                    app.is_loading = false;
                }
                _ => {}
            }
        }

        // 5. Handle Keyboard Input
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Global quit check
                if app.input_mode == InputMode::Normal && key.code == KeyCode::Char('q') {
                     break;
                }

                // If user presses Enter in Normal Mode, send request
                if app.input_mode == InputMode::Normal && key.code == KeyCode::Enter {
                    let _ = ui_tx.send(NetworkEvent::RunRequest(app.url.clone())).await;
                    app.is_loading = true;
                }
                // Handle navigation/typing
                handler::handle_key_events(key, &mut app);
            }
        }
    }

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
