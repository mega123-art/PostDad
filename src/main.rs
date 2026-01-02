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
    
    let (ui_tx, network_rx) = mpsc::channel(32);
    let (network_tx, mut ui_rx) = mpsc::channel(32);

    
    tokio::spawn(async move {
        handle_network(network_rx, network_tx).await;
    });

    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        if app.should_open_editor() {
            
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            terminal.show_cursor()?;

            
            
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
            
            let filename = if app.editor_mode == crate::app::EditorMode::Headers {
                "postdad_headers.json" 
            } else {
                "postdad_body.json" 
            };
            file_path.push(filename);
            
            
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

            
            if let Ok(s) = status {
                if s.success() {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                         if app.editor_mode == crate::app::EditorMode::Headers {
                             
                             if let Ok(headers) = serde_json::from_str::<std::collections::HashMap<String, String>>(&content) {
                                 app.request_headers = headers;
                             }
                         } else {
                             app.request_body = content;
                         }
                    }
                }
            }
            
            
            app.editor_mode = crate::app::EditorMode::None;
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
            terminal.hide_cursor()?;
            terminal.clear()?;
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        
        while let Ok(event) = ui_rx.try_recv() {
            match event {
                NetworkEvent::GotResponse(text, status, duration) => {
                    
                    if let Ok(val) = serde_json::from_str::<Value>(&text) {
                        let root = crate::app::JsonEntry::from_value("root".to_string(), &val, 0);
                        
                        app.response_json = Some(vec![root]);
                    } else {
                        app.response_json = None;
                    }

                    app.response = Some(text);
                    app.latency = Some(duration);
                    app.status_code = Some(status);
                    app.is_loading = false;
                    
                    
                    
                    app.add_history(app.method.clone(), app.process_url(), duration);
                }
                NetworkEvent::Error(e) => {
                    app.response = Some(format!("Error: {}", e));
                    app.is_loading = false;
                }
                _ => {}
            }
        }

        
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                
                if app.input_mode == InputMode::Normal && key.code == KeyCode::Char('q') {
                     break;
                }

                
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
                
                handler::handle_key_events(key, &mut app);
            }
        }
    }

    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
