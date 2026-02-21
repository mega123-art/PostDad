use crate::app::{App, InputMode};
use crate::net::http::{NetworkEvent, handle_network};
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

mod domain;
mod features;
mod handler;
mod net;
mod tests;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize syntax highlighting
    // Initialize syntax highlighting
    ui::syntax::init();

    // Parse CLI arguments
    if let Some(action) = features::cli::parse_args() {
        match action {
            features::cli::CliAction::Import(path) => match features::import::import_auto(&path) {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    eprintln!("Import error: {}", e);
                    std::process::exit(1);
                }
            },
            features::cli::CliAction::ExportPostman(collection, output) => {
                match features::export::export_to_postman(&collection, &output) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Export error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            features::cli::CliAction::ExportInsomnia(collection, output) => {
                match features::export::export_to_insomnia(&collection, &output) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Export error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            features::cli::CliAction::GistSync(token) => {
                match features::gist_sync::sync_to_gist(&token).await {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Sync error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            features::cli::CliAction::Run(args) => {
                let exit_code = features::cli::run_collection_cli(args).await;
                std::process::exit(exit_code);
            }
        }
    }

    // No CLI action - launch TUI
    let (ui_tx, network_rx) = mpsc::channel(32);
    let (network_tx, mut ui_rx) = mpsc::channel(32);

    // WebSocket event channels
    let (ws_event_tx, mut ws_event_rx) = mpsc::channel::<crate::net::websocket::WsEvent>(32);
    let ws_handle = crate::net::websocket::spawn_ws_handler(ws_event_tx);

    // Runner event channel
    let (runner_tx, mut runner_rx) = mpsc::channel::<crate::features::runner::RunnerEvent>(32);

    // Stress event channel
    let (stress_tx, mut stress_rx) = mpsc::channel::<features::stress::StressEvent>(32);

    // Sentinel event channel
    let (sentinel_tx, mut sentinel_rx) = mpsc::channel::<features::sentinel::SentinelResult>(32);

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
        if app.active_tab().is_loading
            && last_spinner_tick.elapsed() > std::time::Duration::from_millis(100)
        {
            app.spinner_state = (app.spinner_state + 1) % 10;
            last_spinner_tick = std::time::Instant::now();
        }

        if app.should_open_editor() {
            let _ = disable_raw_mode();
            let _ = execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            );
            let _ = terminal.show_cursor();

            let editor_var = std::env::var("EDITOR");
            let editor_cmd = if let Ok(e) = editor_var {
                e
            } else if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "nano".to_string()
            };

            let mut file_path = std::env::temp_dir();

            let filename = match app.editor_mode {
                crate::app::EditorMode::Headers => "postdad_headers.json",
                crate::app::EditorMode::GraphQLQuery => "postdad_query.graphql",
                crate::app::EditorMode::GraphQLVariables => "postdad_vars.json",
                crate::app::EditorMode::PreRequestScript => "postdad_script.rhai",
                crate::app::EditorMode::PostRequestScript => "postdad_post_script.rhai",
                _ => "postdad_body.json",
            };
            file_path.push(filename);

            match app.editor_mode {
                crate::app::EditorMode::Headers => {
                    let json = serde_json::to_string_pretty(&app.active_tab().request_headers)?;
                    std::fs::write(&file_path, json)?;
                }
                crate::app::EditorMode::GraphQLQuery => {
                    std::fs::write(&file_path, &app.active_tab().graphql_query)?;
                }
                crate::app::EditorMode::GraphQLVariables => {
                    std::fs::write(&file_path, &app.active_tab().graphql_variables)?;
                }
                crate::app::EditorMode::PreRequestScript => {
                    // Write template if script is empty
                    let content = if app.active_tab().pre_request_script.is_empty() {
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
"#
                        .to_string()
                    } else {
                        app.active_tab().pre_request_script.clone()
                    };
                    std::fs::write(&file_path, content)?;
                }
                crate::app::EditorMode::PostRequestScript => {
                    let content = if app.active_tab().post_request_script.is_empty() {
                        r#"// Post-Requests Script (Test Scripts) - Rhai
// Available functions:
//   test(name, passed) - Record a test result
//   status_code() - Response status code (200, 404, etc)
//   response_time() - Response latency in ms
//   response_body() - Raw response body text
//   get_header(name) - Get response header value
//   json_path(query) - Extract value using JSONPath (e.g. "$.data.id")
//   print(msg) - Debug log
//
// Example:
// test("Status is 200", status_code() == 200);
// test("Rapid Response", response_time() < 500);
// let id = json_path("$.id");
// test("ID exists", id != "");
"#
                        .to_string()
                    } else {
                        app.active_tab().post_request_script.clone()
                    };
                    std::fs::write(&file_path, content)?;
                }
                _ => {
                    std::fs::write(&file_path, &app.active_tab().request_body)?;
                }
            }

            let mut parts = editor_cmd.split_whitespace();
            let command = parts.next().unwrap_or("notepad");
            let args: Vec<&str> = parts.collect();

            let mut cmd = std::process::Command::new(command);
            cmd.args(&args).arg(&file_path);

            let status = cmd.status();

            if let Ok(s) = status
                && s.success()
                && let Ok(content) = std::fs::read_to_string(&file_path)
            {
                let editor_mode = app.editor_mode;
                let tab = app.active_tab_mut();
                match editor_mode {
                    crate::app::EditorMode::Headers => {
                        if let Ok(headers) = serde_json::from_str::<
                            std::collections::HashMap<String, String>,
                        >(&content)
                        {
                            tab.request_headers = headers;
                        }
                    }
                    crate::app::EditorMode::GraphQLQuery => {
                        tab.graphql_query = content;
                    }
                    crate::app::EditorMode::GraphQLVariables => {
                        tab.graphql_variables = content;
                    }
                    crate::app::EditorMode::PreRequestScript => {
                        tab.pre_request_script = content;
                    }
                    crate::app::EditorMode::PostRequestScript => {
                        tab.post_request_script = content;
                    }
                    _ => {
                        tab.request_body = content;
                    }
                }
            }

            app.editor_mode = crate::app::EditorMode::None;
            if let Err(e) = enable_raw_mode() {
                eprintln!("Error restoring raw mode: {}", e);
            }
            if let Err(e) = execute!(
                terminal.backend_mut(),
                EnterAlternateScreen,
                EnableMouseCapture
            ) {
                eprintln!("Error restoring terminal state: {}", e);
            }
            let _ = terminal.hide_cursor();
            let _ = terminal.clear();
        }

        if app.active_tab().trigger_oauth_flow {
            app.active_tab_mut().trigger_oauth_flow = false;
            app.show_notification("Opening Browser... Waiting for callback...".to_string());

            let client_id = app.active_tab().oauth_client_id.clone();
            let auth_url = app.active_tab().oauth_auth_url.clone();
            let tx_clone = ui_tx.clone();

            tokio::spawn(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:54321").await;
                if let Ok(l) = listener {
                    let redirect_uri = "http://localhost:54321";
                    let target = format!(
                        "{}?client_id={}&redirect_uri={}&response_type=code",
                        auth_url, client_id, redirect_uri
                    );

                    if webbrowser::open(&target).is_ok()
                        && let Ok((mut stream, _)) = l.accept().await
                    {
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
            });
        }

        if let Some(time) = app.notification_time
            && time.elapsed() > std::time::Duration::from_secs(3)
        {
            app.popup_message = None;
            app.notification_time = None;
        }

        terminal.draw(|f| ui::render(f, &mut app))?;

        while let Ok(event) = ui_rx.try_recv() {
            match event {
                NetworkEvent::OAuthCode(code) => {
                    app.show_notification(
                        "Auth Code Received! Exchanging for Token...".to_string(),
                    );
                    let token_url = app.active_tab().oauth_token_url.clone();
                    let client_id = app.active_tab().oauth_client_id.clone();
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

                        if let Ok(resp) = res
                            && let Ok(bytes) = resp.bytes().await
                        {
                            let text_content = String::from_utf8_lossy(&bytes);
                            if let Ok(json) =
                                serde_json::from_str::<serde_json::Value>(&text_content)
                            {
                                if let Some(token) =
                                    json.get("access_token").and_then(|v| v.as_str())
                                {
                                    let _ =
                                        tx2.send(NetworkEvent::OAuthToken(token.to_string())).await;
                                } else {
                                    let _ = tx2
                                        .send(NetworkEvent::Error(format!(
                                            "No access_token in response: {}",
                                            text_content
                                        )))
                                        .await;
                                }
                            } else if let Some(start) = text_content.find("access_token=") {
                                let rem = &text_content[start + 13..];
                                let end = rem.find('&').unwrap_or(rem.len());
                                let token = &rem[..end];
                                let _ = tx2.send(NetworkEvent::OAuthToken(token.to_string())).await;
                            } else {
                                let _ = tx2
                                    .send(NetworkEvent::Error(format!(
                                        "OAuth exchange failed: {}",
                                        text_content
                                    )))
                                    .await;
                            }
                        }
                    });
                }
                NetworkEvent::OAuthToken(token) => {
                    let tab = app.active_tab_mut();
                    tab.auth_token = token;
                    tab.auth_type = crate::app::AuthType::Bearer; // Switch to Bearer mode with new token
                    app.show_notification("OAuth Success! Token obtained.".to_string());
                }
                NetworkEvent::GotResponse(
                    bytes,
                    status,
                    duration,
                    cookies,
                    resp_url,
                    resp_headers,
                ) => {
                    app.add_cookies(&resp_url, cookies);

                    // Try to decode as UTF-8
                    let text_opt = String::from_utf8(bytes.clone()).ok();
                    let is_binary = text_opt.is_none();
                    let text_display = text_opt
                        .clone()
                        .unwrap_or_else(|| "[Binary Content]".to_string());

                    // Scoped block for extracting variables to avoid mutable borrow conflict
                    // Only try to extract vars if it looks like text (JSON likely)
                    if let Some(text_content) = &text_opt {
                        let val_opt = serde_json::from_str::<Value>(text_content).ok();
                        if let Some(val) = &val_opt
                            && !app.active_tab().extract_rules.is_empty()
                            && !app.environments.is_empty()
                        {
                            let env_idx = app.selected_env_index;
                            // We need to clone extract rules to avoid borrowing app.active_tab() while mutating app (environments)
                            let rules = app.active_tab().extract_rules.clone();

                            if let Some(env) = app.environments.get_mut(env_idx) {
                                for (var_name, path) in rules {
                                    let path_str = if path.starts_with('$') {
                                        path
                                    } else {
                                        format!("$.{}", path)
                                    };

                                    if let Ok(matches) = json_select(val, &path_str)
                                        && let Some(match_val) = matches.first()
                                    {
                                        let val_str = match match_val {
                                            Value::String(s) => s.clone(),
                                            Value::Number(n) => n.to_string(),
                                            Value::Bool(b) => b.to_string(),
                                            _ => match_val.to_string(),
                                        };
                                        env.variables.insert(var_name, val_str);
                                    }
                                }
                            }
                        }
                    }

                    {
                        let tab = app.active_tab_mut();
                        tab.response_json = None;

                        if let Some(text_content) = &text_opt
                            && let Ok(val) = serde_json::from_str::<Value>(text_content)
                        {
                            let root =
                                crate::app::JsonEntry::from_value("root".to_string(), &val, 0);
                            tab.response_json = Some(vec![root]);
                        }

                        tab.response = Some(text_display.clone());
                        tab.response_bytes = Some(bytes.clone()); // Store raw bytes
                        tab.response_is_binary = is_binary;
                        tab.response_image = None;

                        if is_binary && let Ok(img) = image::load_from_memory(&bytes) {
                            tab.response_image = Some(img);
                        }
                        tab.response_headers = resp_headers.clone();

                        tab.latency = Some(duration);
                        tab.status_code = Some(status);
                        tab.is_loading = false;

                        tab.latency_history.push(duration as u64);
                        if tab.latency_history.len() > 40 {
                            tab.latency_history.remove(0);
                        }
                    }

                    // Run Post-Request Script (Only if text)
                    if let Some(text_content) = &text_opt {
                        let script_content = app.active_tab().post_request_script.clone();

                        if !script_content.trim().is_empty() {
                            let result = crate::features::scripting::run_post_script(
                                &script_content,
                                status,
                                text_content,
                                &resp_headers,
                                duration,
                            );
                            let tab = app.active_tab_mut();
                            tab.test_results = result.tests;
                            for e in result.errors {
                                tab.script_output.push(e);
                            }
                        } else {
                            app.active_tab_mut().test_results.clear();
                        }
                    }

                    let method = app.active_tab().method.clone();
                    let url = app.process_url();
                    app.add_history(
                        method,
                        url,
                        duration,
                        status,
                        Some(text_display),
                        resp_headers,
                        Some(bytes),
                        is_binary,
                    );
                }
                NetworkEvent::Error(e) => {
                    let tab = app.active_tab_mut();
                    tab.response = Some(format!("Error: {}", e));
                    tab.status_code = None; // Ensure no status code is shown
                    tab.is_loading = false;
                }
                NetworkEvent::GotSchema(json) => {
                    app.parse_schema_json(&json);
                }
                NetworkEvent::GotGrpcResponse {
                    success,
                    body,
                    error,
                    latency_ms,
                } => {
                    let tab = app.active_tab_mut();
                    tab.is_loading = false;
                    tab.latency = Some(latency_ms);
                    tab.latency_history.push(latency_ms as u64);

                    if success {
                        tab.status_code = Some(0); // gRPC OK is code 0
                        tab.response = Some(body.clone());

                        // Try to parse as JSON for the explorer
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                            let entries = vec![crate::app::JsonEntry::from_value(
                                "root".to_string(),
                                &parsed,
                                0,
                            )];
                            tab.response_json = Some(entries);
                        }

                        app.show_notification(format!("gRPC OK ({} ms)", latency_ms));
                    } else {
                        tab.status_code = Some(1); // gRPC error
                        let error_msg = error.unwrap_or_else(|| "Unknown gRPC error".to_string());
                        tab.response =
                            Some(format!("gRPC Error:\n{}\n\nResponse:\n{}", error_msg, body));
                        app.show_notification("gRPC Error".to_string());
                    }
                }
                NetworkEvent::GotGrpcServices(services) => {
                    let tab = app.active_tab_mut();
                    tab.grpc_services = services;
                    tab.show_grpc_services_modal = true;
                    app.show_notification("Services discovered via reflection".to_string());
                }
                NetworkEvent::GotGrpcServiceDescription(desc) => {
                    let tab = app.active_tab_mut();
                    tab.grpc_service_description = desc;
                    tab.show_grpc_description_modal = true;
                    tab.show_grpc_services_modal = false; // Close services modal
                    app.show_notification("Service description loaded".to_string());
                }
                _ => {}
            }
        }

        // Handle WebSocket events
        while let Ok(ws_event) = ws_event_rx.try_recv() {
            match ws_event {
                crate::net::websocket::WsEvent::Connected => {
                    app.active_tab_mut().ws_connected = true;
                    app.show_notification("WebSocket Connected!".to_string());
                }
                crate::net::websocket::WsEvent::Disconnected => {
                    app.active_tab_mut().ws_connected = false;
                    app.show_notification("WebSocket Disconnected".to_string());
                }
                crate::net::websocket::WsEvent::Message(msg) => {
                    let tab = app.active_tab_mut();
                    tab.ws_messages.push(crate::net::websocket::WsMessage {
                        content: msg,
                        is_sent: false,
                        timestamp: std::time::Instant::now(),
                    });
                    // Keep message history limited
                    if tab.ws_messages.len() > 100 {
                        tab.ws_messages.remove(0);
                    }
                }
                crate::net::websocket::WsEvent::Error(e) => {
                    app.show_notification(format!("WS Error: {}", e));
                }
            }
        }

        // Handle Runner events
        while let Ok(runner_event) = runner_rx.try_recv() {
            match runner_event {
                crate::features::runner::RunnerEvent::Started {
                    collection_name,
                    total,
                } => {
                    app.runner_result = Some(crate::features::runner::CollectionRunResult::new(
                        &collection_name,
                        total,
                    ));
                    app.show_notification(format!(
                        "Running {} ({} requests)...",
                        collection_name, total
                    ));
                }
                crate::features::runner::RunnerEvent::RequestStarted { name: _name, index } => {
                    // Update current progress
                    if let Some(ref mut result) = app.runner_result {
                        result.current_index = index;
                    }
                }
                crate::features::runner::RunnerEvent::RequestCompleted(run_result) => {
                    if let Some(ref mut result) = app.runner_result {
                        result.add_result(run_result);
                    }
                }
                crate::features::runner::RunnerEvent::Finished(final_result) => {
                    let passed = final_result.passed;
                    let failed = final_result.failed;
                    let total = final_result.total;
                    app.runner_result = Some(final_result);
                    app.show_notification(format!(
                        "Run Complete: {}/{} passed, {} failed",
                        passed, total, failed
                    ));
                }
                crate::features::runner::RunnerEvent::Error(e) => {
                    app.show_notification(format!("Runner Error: {}", e));
                }
            }
        }

        // Handle Stress events
        while let Ok(stress_event) = stress_rx.try_recv() {
            match stress_event {
                crate::features::stress::StressEvent::Progress {
                    requests_done,
                    elapsed_secs,
                } => {
                    app.stress_progress = Some((requests_done, elapsed_secs));
                }
                crate::features::stress::StressEvent::Finished(stats) => {
                    app.stress_running = false;
                    app.stress_stats = Some(stats);
                    app.show_notification("Stress Test Completed".to_string());
                }
                crate::features::stress::StressEvent::Error(e) => {
                    app.stress_running = false;
                    app.show_notification(format!("Stress Test Failed: {}", e));
                }
            }
        }

        // Handle Sentinel events
        while let Ok(sentinel_res) = sentinel_rx.try_recv() {
            if let Some(state) = &mut app.sentinel_state {
                state.add_result(sentinel_res);
            }
        }

        if event::poll(std::time::Duration::from_millis(16))? {
            let event = event::read()?;
            match event {
                Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }

                    // Handle Sentinel Start
                    if app.should_start_sentinel {
                        app.should_start_sentinel = false;

                        // Create config first (immutable borrow)
                        // Create config first (immutable borrow)
                        let tab = app.active_tab();
                        let mut failure_keyword = None;
                        let mut headers = Vec::new();

                        for (k, v) in &tab.request_headers {
                            if k.eq_ignore_ascii_case("X-Fail-If") {
                                failure_keyword = Some(v.clone());
                            } else {
                                headers.push((k.clone(), v.clone()));
                            }
                        }

                        let interval_val = app.sentinel_interval_input.parse::<u64>().unwrap_or(2);
                        let config = crate::features::sentinel::SentinelConfig {
                            url: app.process_url(),
                            method: tab.method.clone(),
                            headers,
                            body: if !tab.request_body.is_empty() {
                                Some(tab.request_body.clone())
                            } else {
                                None
                            },
                            interval_secs: interval_val,
                            failure_keyword,
                        };

                        // Then mutate state
                        if let Some(state) = &mut app.sentinel_state {
                            let tx = sentinel_tx.clone();
                            let (stop_tx, stop_rx) = tokio::sync::mpsc::channel(1);
                            state.stop_tx = Some(stop_tx);
                            state.is_running = true;

                            tokio::spawn(crate::features::sentinel::run_sentinel_task(
                                config, tx, stop_rx,
                            ));
                        }
                    }

                    // Handle Stress Test Trigger
                    if app.should_run_stress_test {
                        app.should_run_stress_test = false;
                        app.stress_running = true;
                        app.stress_stats = None;
                        app.stress_progress = None;

                        let tab = app.active_tab();
                        let vus = app.stress_vus_input.parse().unwrap_or(50);
                        let duration = app.stress_duration_input.parse().unwrap_or(10);

                        let config = crate::features::stress::StressConfig {
                            url: app.process_url(),
                            method: tab.method.clone(),
                            headers: tab.request_headers.clone(), // Note: Auth handling skipped for brevity, user should set headers
                            body: if !tab.request_body.is_empty() {
                                Some(tab.request_body.clone())
                            } else {
                                None
                            },
                            concurrency: vus,
                            duration_secs: duration,
                        };

                        let tx = stress_tx.clone();
                        app.show_notification(format!(
                            "Starting Stress Test ({} VUs, {}s)...",
                            vus, duration
                        ));
                        tokio::spawn(crate::features::stress::run_stress_test(config, tx));
                    }

                    if app.active_tab().should_introspect_schema {
                        app.active_tab_mut().should_introspect_schema = false;

                        let tab = app.active_tab();
                        let url = tab.url.clone();
                        let mut headers = tab.request_headers.clone(); // Basic headers

                        // Add Auth header if needed - leveraging existing auth logic would be better but simple manual construction for now
                        // Note: This duplicates some auth logic from network.rs but we need to pass headers to the event
                        // Actually network.rs handles auth payloads for RunRequest, but IntrospectSchema takes raw headers.
                        // We should probably just pass AuthPayload to IntrospectSchema too?
                        // To keep it simple, let's manual auth here OR update NetworkEvent.

                        // Simple manual Auth header construction
                        match &tab.auth_type {
                            crate::app::AuthType::Bearer => {
                                headers.insert(
                                    "Authorization".to_string(),
                                    format!("Bearer {}", tab.auth_token),
                                );
                            }
                            crate::app::AuthType::Basic => {
                                use base64::prelude::*;
                                let auth =
                                    format!("{}:{}", tab.basic_auth_user, tab.basic_auth_pass);
                                let encoded = BASE64_STANDARD.encode(auth);
                                headers.insert(
                                    "Authorization".to_string(),
                                    format!("Basic {}", encoded),
                                );
                            }
                            crate::app::AuthType::OAuth2 => {
                                headers.insert(
                                    "Authorization".to_string(),
                                    format!("Bearer {}", tab.auth_token),
                                );
                            }
                            _ => {}
                        }

                        let _ = ui_tx
                            .send(NetworkEvent::IntrospectSchema { url, headers })
                            .await;
                        app.show_notification("Introspecting Schema...".to_string());
                    }

                    // Handle should_list_grpc_services flag
                    if app.active_tab().should_list_grpc_services {
                        app.active_tab_mut().should_list_grpc_services = false;

                        let tab = app.active_tab();
                        let url = tab
                            .url
                            .clone()
                            .replace("https://", "")
                            .replace("http://", "")
                            .replace("grpc://", "");
                        let use_plaintext = !tab.url.starts_with("https://");

                        let _ = ui_tx
                            .send(NetworkEvent::ListGrpcServices { url, use_plaintext })
                            .await;
                        app.show_notification("Discovering gRPC services...".to_string());
                    }

                    // Handle should_describe_grpc_service flag
                    if app.active_tab().should_describe_grpc_service {
                        app.active_tab_mut().should_describe_grpc_service = false;

                        let tab = app.active_tab();
                        let url = tab
                            .url
                            .clone()
                            .replace("https://", "")
                            .replace("http://", "")
                            .replace("grpc://", "");
                        let service = tab.grpc_service_to_describe.clone();
                        let use_plaintext = !tab.url.starts_with("https://");

                        let _ = ui_tx
                            .send(NetworkEvent::DescribeGrpcService {
                                url,
                                service,
                                use_plaintext,
                            })
                            .await;
                        app.show_notification("Fetching service description...".to_string());
                    }

                    if app.active_tab().input_mode == InputMode::Normal
                        && key.code == KeyCode::Char('q')
                    {
                        break;
                    }

                    // Runner mode: Enter to run selected collection
                    if app.runner_mode
                        && app.active_tab().input_mode == InputMode::Normal
                        && key.code == KeyCode::Enter
                    {
                        // Check if a run is already in progress
                        if let Some(ref result) = app.runner_result
                            && result.running
                        {
                            app.show_notification("Run already in progress...".to_string());
                            handler::handle_key_events(key, &mut app);
                            continue;
                        }

                        // Get selected collection
                        if let Some(idx) = app.collection_state.selected()
                            && idx < app.collections.len()
                        {
                            let collection = app.collections[idx].clone();
                            let env_vars = if !app.environments.is_empty() {
                                app.environments[app.selected_env_index].variables.clone()
                            } else {
                                std::collections::HashMap::new()
                            };

                            let runner_tx_clone = runner_tx.clone();
                            app.runner_scroll = 0;

                            tokio::spawn(async move {
                                crate::features::runner::run_collection(
                                    &collection,
                                    &env_vars,
                                    runner_tx_clone,
                                )
                                .await;
                            });
                        }
                        handler::handle_key_events(key, &mut app);
                        continue;
                    }

                    // WebSocket mode: Enter to connect/disconnect, send message when in message input mode
                    if app.active_tab().app_mode == crate::app::AppMode::WebSocket {
                        if app.active_tab().input_mode == InputMode::EditingWsMessage
                            && key.code == KeyCode::Enter
                        {
                            // Send message
                            let msg = app.active_tab().ws_message_input.clone();
                            let connected = app.active_tab().ws_connected;

                            if !msg.is_empty() && connected {
                                let tab = app.active_tab_mut();
                                tab.ws_messages.push(crate::net::websocket::WsMessage {
                                    content: msg.clone(),
                                    is_sent: true,
                                    timestamp: std::time::Instant::now(),
                                });
                                let _ = ws_handle
                                    .command_tx
                                    .send(crate::net::websocket::WsCommand::Send(msg))
                                    .await;
                                app.active_tab_mut().ws_message_input.clear();
                            }
                        } else if app.active_tab().input_mode == InputMode::Normal
                            && key.code == KeyCode::Enter
                        {
                            // Connect or disconnect
                            if app.active_tab().ws_connected {
                                let _ = ws_handle
                                    .command_tx
                                    .send(crate::net::websocket::WsCommand::Disconnect)
                                    .await;
                            } else {
                                let url = app.active_tab().ws_url.clone();
                                let _ = ws_handle
                                    .command_tx
                                    .send(crate::net::websocket::WsCommand::Connect(url))
                                    .await;
                            }
                        }
                        handler::handle_key_events(key, &mut app);
                        continue;
                    }

                    // HTTP mode: Enter to send request
                    if app.active_tab().input_mode == InputMode::Normal
                        && key.code == KeyCode::Enter
                    {
                        let processed_url = app.process_url();
                        let tab = app.active_tab();

                        let body = if tab.body_type == crate::app::BodyType::Raw
                            && !tab.request_body.trim().is_empty()
                        {
                            Some(tab.request_body.clone())
                        } else if tab.body_type == crate::app::BodyType::GraphQL {
                            let vars: serde_json::Value = if tab.graphql_variables.trim().is_empty()
                            {
                                serde_json::json!({})
                            } else {
                                serde_json::from_str(&tab.graphql_variables)
                                    .unwrap_or(serde_json::json!({}))
                            };
                            let payload = serde_json::json!({
                                "query": tab.graphql_query,
                                "variables": vars
                            });
                            Some(payload.to_string())
                        } else {
                            None
                        };

                        let form_data = if tab.body_type == crate::app::BodyType::FormData
                            && !tab.form_data.is_empty()
                        {
                            Some(tab.form_data.clone())
                        } else {
                            None
                        };

                        let auth = match tab.auth_type {
                            crate::app::AuthType::Bearer => {
                                if !tab.auth_token.is_empty() {
                                    Some(crate::net::http::AuthPayload::Bearer(
                                        tab.auth_token.clone(),
                                    ))
                                } else {
                                    None
                                }
                            }
                            crate::app::AuthType::Basic => {
                                if !tab.basic_auth_user.is_empty()
                                    || !tab.basic_auth_pass.is_empty()
                                {
                                    Some(crate::net::http::AuthPayload::Basic(
                                        tab.basic_auth_user.clone(),
                                        tab.basic_auth_pass.clone(),
                                    ))
                                } else {
                                    None
                                }
                            }
                            crate::app::AuthType::None => None,
                            crate::app::AuthType::OAuth2 => {
                                if !tab.auth_token.is_empty() {
                                    Some(crate::net::http::AuthPayload::Bearer(
                                        tab.auth_token.clone(),
                                    ))
                                } else {
                                    None
                                }
                            }
                        };

                        let mut final_headers = tab.request_headers.clone();
                        // We need to drop tab reference to call app.get_cookie_header which borrows app
                        // But tab reference is used for auth loops above? No, we cloned relevant data
                        // wait, tab is borrowing app.

                        // Actually, let's just clone headers and auth and stuff early, avoiding long borrows.
                        // The above code uses `tab` which is &RequestTab.
                        // `app.get_cookie_header` takes `&self`. This might be okay if `tab` is not mut.
                        // But `app` is borrowed immutably by `tab`, so `app.get_cookie_header` (immutable borrow) is fine.

                        if let Some(cookie_header) = app.get_cookie_header(&processed_url) {
                            final_headers.insert("Cookie".to_string(), cookie_header);
                        }

                        // Run pre-request script
                        let mut final_url = processed_url.clone();
                        let mut final_body = body.clone();
                        app.active_tab_mut().script_output.clear();

                        if !app.active_tab().pre_request_script.trim().is_empty() {
                            let env_vars: std::collections::HashMap<String, String> =
                                if !app.environments.is_empty() {
                                    app.environments[app.selected_env_index].variables.clone()
                                } else {
                                    std::collections::HashMap::new()
                                };

                            // Need to clone script content to avoid borrow issues
                            let script_content = app.active_tab().pre_request_script.clone();
                            let method = app.active_tab().method.clone();

                            let script_result = crate::features::scripting::run_script(
                                &script_content,
                                &method,
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
                                    app.environments[app.selected_env_index]
                                        .variables
                                        .insert(k.clone(), v.clone());
                                }
                            }

                            if let Some(new_body) = script_result.body_override {
                                final_body = Some(new_body);
                            }

                            if let Some(new_url) = script_result.url_override {
                                final_url = new_url;
                            }

                            // Store script output for display
                            app.active_tab_mut().script_output = script_result.errors;
                        }

                        // Check if this is a gRPC request
                        if app.active_tab().body_type == crate::app::BodyType::Grpc {
                            let tab = app.active_tab();
                            let url = tab.url.clone();
                            // Combine service and method if method is set separately
                            let service_method = if tab.grpc_method.is_empty() {
                                tab.grpc_service.clone()
                            } else {
                                format!("{}/{}", tab.grpc_service, tab.grpc_method)
                            };
                            let proto_path = if tab.grpc_proto_path.is_empty() {
                                None
                            } else {
                                Some(tab.grpc_proto_path.clone())
                            };
                            let payload = tab.request_body.clone();
                            let headers = tab.request_headers.clone();

                            // Determine if plaintext based on URL scheme
                            let use_plaintext = !url.starts_with("https://");

                            // Strip scheme for grpcurl (it expects just host:port)
                            let grpc_addr = url
                                .replace("https://", "")
                                .replace("http://", "")
                                .replace("grpc://", "");

                            let _ = ui_tx
                                .send(NetworkEvent::RunGrpc {
                                    url: grpc_addr,
                                    service_method,
                                    proto_path,
                                    payload,
                                    headers,
                                    use_plaintext,
                                })
                                .await;
                            app.active_tab_mut().clear_response();
                            app.active_tab_mut().is_loading = true;
                        } else {
                            // Regular HTTP request
                            let method = app.active_tab().method.clone();
                            let timeout = app.active_tab().timeout_ms;

                            // Load SSL certificates from paths
                            let ssl_ca_cert = app
                                .ssl_ca_cert_path
                                .as_ref()
                                .and_then(|p| std::fs::read(p).ok());
                            let ssl_client_cert = app
                                .ssl_client_cert_path
                                .as_ref()
                                .and_then(|p| std::fs::read(p).ok());
                            let ssl_client_key = app
                                .ssl_client_key_path
                                .as_ref()
                                .and_then(|p| std::fs::read(p).ok());

                            // Prepare proxy authentication if both user and pass are set
                            let proxy_auth = match (&app.proxy_auth_user, &app.proxy_auth_pass) {
                                (Some(user), Some(pass)) => Some((user.clone(), pass.clone())),
                                _ => None,
                            };

                            let _ = ui_tx
                                .send(NetworkEvent::RunRequest {
                                    url: final_url,
                                    method,
                                    headers: final_headers,
                                    body: final_body,
                                    form_data,
                                    auth,
                                    timeout_ms: Some(timeout),
                                    ssl_verify: app.ssl_verify,
                                    ssl_ca_cert,
                                    ssl_client_cert,
                                    ssl_client_key,
                                    proxy_url: app.proxy_url.clone(),
                                    proxy_auth,
                                    no_proxy: app.no_proxy.clone(),
                                })
                                .await;
                            app.active_tab_mut().clear_response();
                            app.active_tab_mut().is_loading = true;
                        }
                    }

                    handler::handle_key_events(key, &mut app);
                }
                Event::Mouse(mouse_event) => {
                    handler::handle_mouse_event(mouse_event, &mut app);
                }
                _ => {}
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
