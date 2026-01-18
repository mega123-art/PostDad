use crate::app::{App, InputMode};
use crate::network::{NetworkEvent, handle_network};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use jsonpath_lib::select as json_select;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use serde_json::Value;
use std::io;
use tokio::sync::mpsc;
mod app;
mod collection;
mod environment;
mod handler;
mod network;
mod runner;
mod scripting;
mod ui;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (ui_tx, network_rx) = mpsc::channel(32);
    let (network_tx, mut ui_rx) = mpsc::channel(32);

    // WebSocket event channels
    let (ws_event_tx, mut ws_event_rx) = mpsc::channel::<websocket::WsEvent>(32);
    let ws_handle = websocket::spawn_ws_handler(ws_event_tx);

    // Runner event channel
    let (runner_tx, mut runner_rx) = mpsc::channel::<runner::RunnerEvent>(32);

    tokio::spawn(async move {
        handle_network(network_rx, network_tx).await;
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut last_spinner_tick = std::time::Instant::now();

    loop {
        if app.is_loading && last_spinner_tick.elapsed() > std::time::Duration::from_millis(100) {
            app.spinner_state = (app.spinner_state + 1) % 10;
            last_spinner_tick = std::time::Instant::now();
        }

        if app.should_open_editor() {
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
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

            let filename = match app.editor_mode {
                crate::app::EditorMode::Headers => "postdad_headers.json",
                crate::app::EditorMode::GraphQLQuery => "postdad_query.graphql",
                crate::app::EditorMode::GraphQLVariables => "postdad_vars.json",
                crate::app::EditorMode::PreRequestScript => "postdad_script.rhai",
                _ => "postdad_body.json",
            };
            file_path.push(filename);

            match app.editor_mode {
                crate::app::EditorMode::Headers => {
                    let json = serde_json::to_string_pretty(&app.request_headers)?;
                    std::fs::write(&file_path, json)?;
                }
                crate::app::EditorMode::GraphQLQuery => {
                    std::fs::write(&file_path, &app.graphql_query)?;
                }
                crate::app::EditorMode::GraphQLVariables => {
                    std::fs::write(&file_path, &app.graphql_variables)?;
                }
                crate::app::EditorMode::PreRequestScript => {
                    // Write template if script is empty
                    let content = if app.pre_request_script.is_empty() {
                        r#"// Pre-Request Script (Rhai)
// Available functions:
//   set_header(name, value) - Add/modify header
//   get_header(name) - Get header value
//   set_var(name, value) - Set variable
//   get_var(name) - Get variable
//   set_body(body) - Override request body
//   set_url(url) - Override request URL
//   timestamp() - Unix timestamp (seconds)
//   timestamp_ms() - Unix timestamp (milliseconds)
//   uuid() - Generate UUID v4
//   base64_encode(text) - Encode as Base64
//   base64_decode(text) - Decode Base64
//   print(msg) - Debug log
// 
// Constants: METHOD, URL, BODY

// Example:
// let ts = timestamp();
// set_header("X-Request-Time", ts.to_string());
"#.to_string()
                    } else {
                        app.pre_request_script.clone()
                    };
                    std::fs::write(&file_path, content)?;
                }
                _ => {
                    std::fs::write(&file_path, &app.request_body)?;
                }
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
                        match app.editor_mode {
                            crate::app::EditorMode::Headers => {
                                if let Ok(headers) = serde_json::from_str::<
                                    std::collections::HashMap<String, String>,
                                >(&content)
                                {
                                    app.request_headers = headers;
                                }
                            }
                            crate::app::EditorMode::GraphQLQuery => {
                                app.graphql_query = content;
                            }
                            crate::app::EditorMode::GraphQLVariables => {
                                app.graphql_variables = content;
                            }
                            crate::app::EditorMode::PreRequestScript => {
                                app.pre_request_script = content;
                            }
                            _ => {
                                app.request_body = content;
                            }
                        }
                    }
                }
            }

            app.editor_mode = crate::app::EditorMode::None;
            enable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                EnterAlternateScreen,
                EnableMouseCapture
            )?;
            terminal.hide_cursor()?;
            terminal.clear()?;
        }

        if app.trigger_oauth_flow {
            app.trigger_oauth_flow = false;
            app.show_notification("Opening Browser... Waiting for callback...".to_string());

            let client_id = app.oauth_client_id.clone();
            let auth_url = app.oauth_auth_url.clone();
            let tx_clone = ui_tx.clone();

            tokio::spawn(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:54321").await;
                if let Ok(l) = listener {
                    let redirect_uri = "http://localhost:54321";
                    let target = format!(
                        "{}?client_id={}&redirect_uri={}&response_type=code",
                        auth_url, client_id, redirect_uri
                    );

                    if webbrowser::open(&target).is_ok() {
                        if let Ok((mut stream, _)) = l.accept().await {
                            use tokio::io::{AsyncReadExt, AsyncWriteExt};
                            let mut buffer = [0; 1024];
                            let _ = stream.read(&mut buffer).await;
                            let request = String::from_utf8_lossy(&buffer);

                            if let Some(start) = request.find("code=") {
                                let remaining = &request[start + 5..];
                                let end = remaining
                                    .find(' ')
                                    .or_else(|| remaining.find('&'))
                                    .unwrap_or(remaining.len());
                                let code = &remaining[..end];

                                let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>PostDad: Auth Successful!</h1><p>You can close this window.</p><script>window.close()</script></body></html>";
                                let _ = stream.write_all(response.as_bytes()).await;
                                let _ = stream.flush().await;

                                let _ = tx_clone
                                    .send(NetworkEvent::OAuthCode(code.to_string()))
                                    .await;
                            }
                        }
                    }
                }
            });
        }

        if let Some(time) = app.notification_time {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                app.popup_message = None;
                app.notification_time = None;
            }
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        while let Ok(event) = ui_rx.try_recv() {
            match event {
                NetworkEvent::OAuthCode(code) => {
                    app.show_notification(
                        "Auth Code Received! Exchanging for Token...".to_string(),
                    );
                    let token_url = app.oauth_token_url.clone();
                    let client_id = app.oauth_client_id.clone();
                    let _redirect_uri = "http://localhost:54321".to_string();

                    let tx2 = ui_tx.clone();
                    tokio::spawn(async move {
                        let client = reqwest::Client::new();
                        let body_str = format!(
                            "client_id={}&code={}&grant_type=authorization_code",
                            client_id, code
                        );

                        let res = client
                            .post(&token_url)
                            .header("Content-Type", "application/x-www-form-urlencoded")
                            .header("Accept", "application/json")
                            .body(body_str)
                            .send()
                            .await;

                        if let Ok(resp) = res {
                            if let Ok(bytes) = resp.bytes().await {
                                let text_content = String::from_utf8_lossy(&bytes);
                                if let Ok(json) =
                                    serde_json::from_str::<serde_json::Value>(&text_content)
                                {
                                    if let Some(token) =
                                        json.get("access_token").and_then(|v| v.as_str())
                                    {
                                        let _ = tx2
                                            .send(NetworkEvent::OAuthToken(token.to_string()))
                                            .await;
                                    } else {
                                        let _ = tx2
                                            .send(NetworkEvent::Error(format!(
                                                "No access_token in response: {}",
                                                text_content
                                            )))
                                            .await;
                                    }
                                } else {
                                    if let Some(start) = text_content.find("access_token=") {
                                        let rem = &text_content[start + 13..];
                                        let end = rem.find('&').unwrap_or(rem.len());
                                        let token = &rem[..end];
                                        let _ = tx2
                                            .send(NetworkEvent::OAuthToken(token.to_string()))
                                            .await;
                                    } else {
                                        let _ = tx2
                                            .send(NetworkEvent::Error(format!(
                                                "OAuth exchange failed: {}",
                                                text_content
                                            )))
                                            .await;
                                    }
                                }
                            }
                        }
                    });
                }
                NetworkEvent::OAuthToken(token) => {
                    app.auth_token = token;
                    app.auth_type = crate::app::AuthType::Bearer; // Switch to Bearer mode with new token
                    app.show_notification("OAuth Success! Token obtained.".to_string());
                }
                NetworkEvent::GotResponse(text, status, duration, cookies, resp_url) => {
                    app.add_cookies(&resp_url, cookies);

                    if let Ok(val) = serde_json::from_str::<Value>(&text) {
                        let root = crate::app::JsonEntry::from_value("root".to_string(), &val, 0);
                        app.response_json = Some(vec![root]);

                        if !app.extract_rules.is_empty() && !app.environments.is_empty() {
                            let env_idx = app.selected_env_index;
                            if let Some(env) = app.environments.get_mut(env_idx) {
                                for (var_name, path) in &app.extract_rules {
                                    let path_str = if path.starts_with('$') {
                                        path.clone()
                                    } else {
                                        format!("$.{}", path)
                                    };

                                    if let Ok(matches) = json_select(&val, &path_str) {
                                        if let Some(match_val) = matches.first() {
                                            let val_str = match match_val {
                                                Value::String(s) => s.clone(),
                                                Value::Number(n) => n.to_string(),
                                                Value::Bool(b) => b.to_string(),
                                                _ => match_val.to_string(),
                                            };
                                            env.variables.insert(var_name.clone(), val_str);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        app.response_json = None;
                    }

                    app.response = Some(text.clone());
                    app.latency = Some(duration);
                    app.status_code = Some(status);
                    app.is_loading = false;

                    app.latency_history.push(duration as u64);
                    if app.latency_history.len() > 40 {
                        app.latency_history.remove(0);
                    }

                    app.add_history(
                        app.method.clone(),
                        app.process_url(),
                        duration,
                        status,
                        Some(text.clone()),
                    );
                }
                NetworkEvent::Error(e) => {
                    app.response = Some(format!("Error: {}", e));
                    app.is_loading = false;
                }
                _ => {}
            }
        }

        // Handle WebSocket events
        while let Ok(ws_event) = ws_event_rx.try_recv() {
            match ws_event {
                websocket::WsEvent::Connected => {
                    app.ws_connected = true;
                    app.show_notification("WebSocket Connected!".to_string());
                }
                websocket::WsEvent::Disconnected => {
                    app.ws_connected = false;
                    app.show_notification("WebSocket Disconnected".to_string());
                }
                websocket::WsEvent::Message(msg) => {
                    app.ws_messages.push(websocket::WsMessage {
                        content: msg,
                        is_sent: false,
                        timestamp: std::time::Instant::now(),
                    });
                    // Keep message history limited
                    if app.ws_messages.len() > 100 {
                        app.ws_messages.remove(0);
                    }
                }
                websocket::WsEvent::Error(e) => {
                    app.show_notification(format!("WS Error: {}", e));
                }
            }
        }

        // Handle Runner events
        while let Ok(runner_event) = runner_rx.try_recv() {
            match runner_event {
                runner::RunnerEvent::Started { collection_name, total } => {
                    app.runner_result = Some(runner::CollectionRunResult::new(&collection_name, total));
                    app.show_notification(format!("Running {} ({} requests)...", collection_name, total));
                }
                runner::RunnerEvent::RequestStarted { name: _name, index } => {
                    // Update current progress
                    if let Some(ref mut result) = app.runner_result {
                        result.current_index = index;
                    }
                }
                runner::RunnerEvent::RequestCompleted(run_result) => {
                    if let Some(ref mut result) = app.runner_result {
                        result.add_result(run_result);
                    }
                }
                runner::RunnerEvent::Finished(final_result) => {
                    let passed = final_result.passed;
                    let failed = final_result.failed;
                    let total = final_result.total;
                    app.runner_result = Some(final_result);
                    app.show_notification(format!(
                        "Run Complete: {}/{} passed, {} failed",
                        passed, total, failed
                    ));
                }
                runner::RunnerEvent::Error(e) => {
                    app.show_notification(format!("Runner Error: {}", e));
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                if app.input_mode == InputMode::Normal && key.code == KeyCode::Char('q') {
                    break;
                }

                // Runner mode: Enter to run selected collection
                if app.runner_mode && app.input_mode == InputMode::Normal && key.code == KeyCode::Enter {
                    // Check if a run is already in progress
                    if let Some(ref result) = app.runner_result {
                        if result.running {
                            app.show_notification("Run already in progress...".to_string());
                            handler::handle_key_events(key, &mut app);
                            continue;
                        }
                    }

                    // Get selected collection
                    if let Some(idx) = app.collection_state.selected() {
                        if idx < app.collections.len() {
                            let collection = app.collections[idx].clone();
                            let env_vars = if !app.environments.is_empty() {
                                app.environments[app.selected_env_index].variables.clone()
                            } else {
                                std::collections::HashMap::new()
                            };

                            let runner_tx_clone = runner_tx.clone();
                            app.runner_scroll = 0;
                            
                            tokio::spawn(async move {
                                runner::run_collection(&collection, &env_vars, runner_tx_clone).await;
                            });
                        }
                    }
                    handler::handle_key_events(key, &mut app);
                    continue;
                }

                // WebSocket mode: Enter to connect/disconnect, send message when in message input mode
                if app.app_mode == crate::app::AppMode::WebSocket {
                    if app.input_mode == InputMode::EditingWsMessage && key.code == KeyCode::Enter {
                        // Send message
                        if !app.ws_message_input.is_empty() && app.ws_connected {
                            let msg = app.ws_message_input.clone();
                            app.ws_messages.push(websocket::WsMessage {
                                content: msg.clone(),
                                is_sent: true,
                                timestamp: std::time::Instant::now(),
                            });
                            let _ = ws_handle.command_tx.send(websocket::WsCommand::Send(msg)).await;
                            app.ws_message_input.clear();
                        }
                    } else if app.input_mode == InputMode::Normal && key.code == KeyCode::Enter {
                        // Connect or disconnect
                        if app.ws_connected {
                            let _ = ws_handle.command_tx.send(websocket::WsCommand::Disconnect).await;
                        } else {
                            let url = app.ws_url.clone();
                            let _ = ws_handle.command_tx.send(websocket::WsCommand::Connect(url)).await;
                        }
                    }
                    handler::handle_key_events(key, &mut app);
                    continue;
                }

                // HTTP mode: Enter to send request
                if app.input_mode == InputMode::Normal && key.code == KeyCode::Enter {
                    let processed_url = app.process_url();
                    let body = if app.body_type == crate::app::BodyType::Raw
                        && !app.request_body.trim().is_empty()
                    {
                        Some(app.request_body.clone())
                    } else if app.body_type == crate::app::BodyType::GraphQL {
                        let vars: serde_json::Value = if app.graphql_variables.trim().is_empty() {
                            serde_json::json!({})
                        } else {
                            serde_json::from_str(&app.graphql_variables)
                                .unwrap_or(serde_json::json!({}))
                        };
                        let payload = serde_json::json!({
                            "query": app.graphql_query,
                            "variables": vars
                        });
                        Some(payload.to_string())
                    } else {
                        None
                    };

                    let form_data = if app.body_type == crate::app::BodyType::FormData
                        && !app.form_data.is_empty()
                    {
                        Some(app.form_data.clone())
                    } else {
                        None
                    };

                    let auth = match app.auth_type {
                        crate::app::AuthType::Bearer => {
                            if !app.auth_token.is_empty() {
                                Some(crate::network::AuthPayload::Bearer(app.auth_token.clone()))
                            } else {
                                None
                            }
                        }
                        crate::app::AuthType::Basic => {
                            if !app.basic_auth_user.is_empty() || !app.basic_auth_pass.is_empty() {
                                Some(crate::network::AuthPayload::Basic(
                                    app.basic_auth_user.clone(),
                                    app.basic_auth_pass.clone(),
                                ))
                            } else {
                                None
                            }
                        }
                        crate::app::AuthType::None => None,
                        crate::app::AuthType::OAuth2 => {
                            if !app.auth_token.is_empty() {
                                Some(crate::network::AuthPayload::Bearer(app.auth_token.clone()))
                            } else {
                                None
                            }
                        }
                    };

                    let mut final_headers = app.request_headers.clone();
                    if let Some(cookie_header) = app.get_cookie_header(&processed_url) {
                        final_headers.insert("Cookie".to_string(), cookie_header);
                    }

                    // Run pre-request script
                    let mut final_url = processed_url.clone();
                    let mut final_body = body.clone();
                    app.script_output.clear();

                    if !app.pre_request_script.trim().is_empty() {
                        let env_vars: std::collections::HashMap<String, String> = if !app.environments.is_empty() {
                            app.environments[app.selected_env_index].variables.clone()
                        } else {
                            std::collections::HashMap::new()
                        };

                        let script_result = scripting::run_script(
                            &app.pre_request_script,
                            &app.method,
                            &final_url,
                            &final_headers,
                            final_body.as_deref().unwrap_or(""),
                            &env_vars,
                        );

                        // Apply script results
                        final_headers = script_result.headers;
                        
                        // Merge script variables back to environment
                        if !app.environments.is_empty() {
                            for (k, v) in &script_result.variables {
                                app.environments[app.selected_env_index].variables.insert(k.clone(), v.clone());
                            }
                        }

                        if let Some(new_body) = script_result.body_override {
                            final_body = Some(new_body);
                        }

                        if let Some(new_url) = script_result.url_override {
                            final_url = new_url;
                        }

                        // Store script output for display
                        app.script_output = script_result.errors;
                    }

                    let _ = ui_tx
                        .send(NetworkEvent::RunRequest {
                            url: final_url,
                            method: app.method.clone(),
                            headers: final_headers,
                            body: final_body,
                            form_data,
                            auth,
                        })
                        .await;
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
