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
mod environment;

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
        if app.should_open_editor() {
            // 1. Suspend TUI
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            terminal.show_cursor()?;

            // 2. Open Editor
            // Cross-platform handling
            let editor_var = std::env::var("EDITOR");
            let editor_cmd = if let Ok(e) = editor_var {
                e
            } else {
                if cfg!(target_os = "windows") {
                    "notepad".to_string()
                } else {
                    "nano".to_string()
                }
            };
            
            let mut file_path = std::env::temp_dir();
            // Determine filename based on mode
            let filename = if app.editor_mode == crate::app::EditorMode::Headers {
                "postdad_headers.json" 
            } else {
                "postdad_body.json" 
            };
            file_path.push(filename);
            
            // write current content to file
            if app.editor_mode == crate::app::EditorMode::Headers {
                let json = serde_json::to_string_pretty(&app.request_headers)?;
                std::fs::write(&file_path, json)?;
            } else {
                std::fs::write(&file_path, &app.request_body)?;
            }

            let mut parts = editor_cmd.split_whitespace();
            let command = parts.next().unwrap_or("notepad");
            let args: Vec<&str> = parts.collect();

            let mut cmd = std::process::Command::new(command);
            cmd.args(&args).arg(&file_path);
            
            let status = cmd.status();

            // 3. Read back
            if let Ok(s) = status {
                if s.success() {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                         if app.editor_mode == crate::app::EditorMode::Headers {
                             // Try parse JSON
                             if let Ok(headers) = serde_json::from_str::<std::collections::HashMap<String, String>>(&content) {
                                 app.request_headers = headers;
                             }
                         } else {
                             app.request_body = content;
                         }
                    }
                }
            }
            
            // 4. Resume TUI
            app.editor_mode = crate::app::EditorMode::None;
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
            terminal.hide_cursor()?;
            terminal.clear()?;
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        // 4. Handle Background Messages (Did the API respond?)
        while let Ok(event) = ui_rx.try_recv() {
            match event {
                NetworkEvent::GotResponse(text, status, duration) => {
                    // Try to parse as JSON for the explorer
                    if let Ok(val) = serde_json::from_str::<Value>(&text) {
                        let root = crate::app::JsonEntry::from_value("root".to_string(), &val, 0);
                        // Wrap in a doc-root if needed or just use a list
                        app.response_json = Some(vec![root]);
                    } else {
                        app.response_json = None;
                    }

                    app.response = Some(text);
                    app.latency = Some(duration);
                    app.status_code = Some(status);
                    app.is_loading = false;
                    
                    // Log to history needs Method / URL. We might need to store "Last Request" in App to do this cleanly
                    // For now, let's just log what we have. It's best if NetworkEvent returned the URL too
                    app.add_history(app.method.clone(), app.process_url(), duration);
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
                    let processed_url = app.process_url();
                    let body = if app.request_body.trim().is_empty() { None } else { Some(app.request_body.clone()) };
                    
                    let _ = ui_tx.send(NetworkEvent::RunRequest { 
                        url: processed_url,
                        method: app.method.clone(),
                        headers: app.request_headers.clone(),
                        body
                    }).await;
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
