use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    if app.show_splash {
        app.show_splash = false;
        return;
    }

    if app.active_tab().show_schema_modal {
        if key_event.code == KeyCode::Esc {
            app.close_schema_modal();
        }
        return;
    }

    if app.active_tab().show_grpc_services_modal {
        match key_event.code {
            KeyCode::Esc => {
                app.active_tab_mut().show_grpc_services_modal = false;
            }
            KeyCode::Enter => {
                // Select the current service and close modal
                if !app.active_tab().grpc_services.is_empty()
                    && let Some(idx) = app.active_tab().form_list_state.selected()
                    && idx < app.active_tab().grpc_services.len()
                {
                    let service = app.active_tab().grpc_services[idx].clone();
                    app.active_tab_mut().grpc_service = service;
                    app.active_tab_mut().show_grpc_services_modal = false;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = app.active_tab().grpc_services.len();
                if len > 0 {
                    let current = app.active_tab().form_list_state.selected().unwrap_or(0);
                    let next = if current >= len - 1 { 0 } else { current + 1 };
                    app.active_tab_mut().form_list_state.select(Some(next));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = app.active_tab().grpc_services.len();
                if len > 0 {
                    let current = app.active_tab().form_list_state.selected().unwrap_or(0);
                    let prev = if current == 0 { len - 1 } else { current - 1 };
                    app.active_tab_mut().form_list_state.select(Some(prev));
                }
            }
            KeyCode::Char('D') | KeyCode::Char('d') => {
                // Describe selected service
                if !app.active_tab().grpc_services.is_empty()
                    && let Some(idx) = app.active_tab().form_list_state.selected()
                    && idx < app.active_tab().grpc_services.len()
                {
                    let service = app.active_tab().grpc_services[idx].clone();
                    app.active_tab_mut().grpc_service_to_describe = service;
                    app.active_tab_mut().should_describe_grpc_service = true;
                }
            }
            _ => {}
        }
        return;
    }

    if app.active_tab().show_grpc_description_modal {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.active_tab_mut().show_grpc_description_modal = false;
            }
            KeyCode::Char('b') => {
                // Go back to services list
                app.active_tab_mut().show_grpc_description_modal = false;
                app.active_tab_mut().show_grpc_services_modal = true;
            }
            _ => {}
        }
        return;
    }

    if app.show_diff_view {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.close_diff();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let i = app.diff_list_state.selected().unwrap_or(0);
                app.diff_list_state.select(Some(i + 1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = app.diff_list_state.selected().unwrap_or(0);
                if i > 0 {
                    app.diff_list_state.select(Some(i - 1));
                }
            }
            _ => {}
        }
        return;
    }

    // Handle help menu scrolling
    if app.show_help {
        match key_event.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                app.show_help = false;
                app.help_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.help_scroll = app.help_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                app.help_scroll = 0; // Go to top
            }
            KeyCode::Char('G') => {
                app.help_scroll = 50; // Go to bottom (approximate)
            }
            _ => {}
        }
    }

    // Cookie Manager Modal
    if app.show_cookie_modal {
        match key_event.code {
            KeyCode::Esc => {
                app.show_cookie_modal = false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let current = app.cookie_list_state.selected().unwrap_or(0);
                let count = app.get_flattened_cookies().len();
                if count > 0 {
                    let next = (current + 1) % count;
                    app.cookie_list_state.select(Some(next));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let current = app.cookie_list_state.selected().unwrap_or(0);
                let count = app.get_flattened_cookies().len();
                if count > 0 {
                    let next = if current == 0 { count - 1 } else { current - 1 };
                    app.cookie_list_state.select(Some(next));
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(selected) = app.cookie_list_state.selected() {
                    app.delete_cookie_at_index(selected);
                    // Adjust selection if needed
                    let count = app.get_flattened_cookies().len();
                    if count == 0 {
                        app.cookie_list_state.select(None);
                    } else if selected >= count {
                        app.cookie_list_state.select(Some(count - 1));
                    }
                }
            }
            _ => {}
        }
        return;
    }

    if app.show_stress_modal {
        match key_event.code {
            KeyCode::Esc => {
                app.show_stress_modal = false;
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                app.show_stress_modal = false;
                app.active_tab_mut().input_mode = InputMode::Normal;
                app.should_run_stress_test = true;
            }

            KeyCode::Tab => {
                if app.active_tab().input_mode == InputMode::EditingStressVUs {
                    app.active_tab_mut().input_mode = InputMode::EditingStressDuration;
                } else {
                    app.active_tab_mut().input_mode = InputMode::EditingStressVUs;
                }
            }
            KeyCode::Char(c) => {
                if app.active_tab().input_mode == InputMode::EditingStressVUs {
                    if c.is_ascii_digit() {
                        app.stress_vus_input.push(c);
                    }
                } else if app.active_tab().input_mode == InputMode::EditingStressDuration
                    && c.is_ascii_digit()
                {
                    app.stress_duration_input.push(c);
                }
            }
            KeyCode::Backspace => {
                if app.active_tab().input_mode == InputMode::EditingStressVUs {
                    app.stress_vus_input.pop();
                } else if app.active_tab().input_mode == InputMode::EditingStressDuration {
                    app.stress_duration_input.pop();
                }
            }
            _ => {}
        }
        return;
    }

    if app.stress_stats.is_some() {
        if key_event.code == KeyCode::Esc {
            app.stress_stats = None;
        }
        return;
    }

    // Handle Sentinel Mode
    if app.sentinel_mode {
        // Handle Input Mode for Interval
        if app.active_tab().input_mode == InputMode::EditingSentinelInterval {
            match key_event.code {
                KeyCode::Esc | KeyCode::Enter => {
                    app.active_tab_mut().input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    if c.is_ascii_digit() {
                        app.sentinel_interval_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    app.sentinel_interval_input.pop();
                }
                _ => {}
            }
            return;
        }

        match key_event.code {
            KeyCode::Esc => app.sentinel_mode = false,
            KeyCode::Char('S') | KeyCode::Char('s') => {
                if let Some(state) = &mut app.sentinel_state {
                    if state.is_running {
                        // Stop it
                        if let Some(_tx) = &state.stop_tx {
                            // channel dropped automatically
                        }
                        state.is_running = false;
                        state.stop_tx = None;
                        app.show_notification("Sentinel Stopped".to_string());
                    } else {
                        // Start it
                        app.should_start_sentinel = true;
                        app.show_notification("Starting Sentinel...".to_string());
                    }
                }
            }
            KeyCode::Char('L') | KeyCode::Char('l') => {
                if let Some(state) = &app.sentinel_state {
                    match state.save_history() {
                        Ok(fname) => app.show_notification(format!("History saved to {}", fname)),
                        Err(e) => app.show_notification(format!("Failed to save: {}", e)),
                    }
                }
            }
            KeyCode::Char('i') => {
                app.sentinel_interval_input.clear();
                app.active_tab_mut().input_mode = InputMode::EditingSentinelInterval;
            }
            _ => {}
        }
        return;
    }

    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            KeyCode::Char('h') => {
                app.active_sidebar = !app.active_sidebar;
                if app.active_sidebar && app.collection_state.selected().is_none() {
                    app.collection_state.select(Some(0));
                }
                return;
            }
            KeyCode::Char('e') => {
                app.next_env();
                return;
            }
            KeyCode::Char('w') => {
                // Toggle between HTTP and WebSocket modes
                let new_mode = match app.active_tab().app_mode {
                    crate::app::AppMode::Http => crate::app::AppMode::WebSocket,
                    crate::app::AppMode::WebSocket => crate::app::AppMode::Http,
                };
                app.active_tab_mut().app_mode = new_mode;
                return;
            }

            KeyCode::Char('r') => {
                // Toggle runner mode
                app.runner_mode = !app.runner_mode;
                if app.runner_mode {
                    // Select first collection if none selected
                    if app.collection_state.selected().is_none() && !app.collections.is_empty() {
                        app.collection_state.select(Some(0));
                    }
                }
                return;
            }
            KeyCode::Char('t') => {
                app.next_theme();
                app.show_notification(format!("Theme: {}", app.theme.name));
                return;
            }
            KeyCode::Char('k') => {
                app.mock_mode = !app.mock_mode;
                return;
            }
            _ => {}
        }
    }

    // Handle Mock Mode
    if app.mock_mode {
        match key_event.code {
            KeyCode::Esc => app.mock_mode = false,
            KeyCode::Char('s') => app.toggle_mock_server(),
            KeyCode::Char('a') => {
                // Add new mock route
                app.mock_routes.push(crate::net::mock_server::MockRoute {
                    path: "/api/new".to_string(),
                    method: "GET".to_string(),
                    status: 200,
                    body: "{\"message\": \"Hello Mock!\"}".to_string(),
                    headers: std::collections::HashMap::new(),
                });
            }
            KeyCode::Char('d') => {
                if let Some(selected) = app.mock_list_state.selected()
                    && selected < app.mock_routes.len()
                {
                    app.mock_routes.remove(selected);
                    app.restart_mock_server_if_running();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let i = match app.mock_list_state.selected() {
                    Some(i) => {
                        if i >= app.mock_routes.len().saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                app.mock_list_state.select(Some(i));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = match app.mock_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            app.mock_routes.len().saturating_sub(1)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                app.mock_list_state.select(Some(i));
            }
            _ => {}
        }
        return;
    }

    // Handle runner mode
    if app.runner_mode {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                // Navigate collections or results
                if let Some(ref result) = app.runner_result {
                    if app.runner_scroll < result.results.len().saturating_sub(1) {
                        app.runner_scroll += 1;
                    }
                } else if !app.collections.is_empty() {
                    let next = match app.collection_state.selected() {
                        Some(i) => {
                            if i >= app.collections.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    app.collection_state.select(Some(next));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(ref _result) = app.runner_result {
                    if app.runner_scroll > 0 {
                        app.runner_scroll -= 1;
                    }
                } else if !app.collections.is_empty() {
                    let prev = match app.collection_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                app.collections.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    app.collection_state.select(Some(prev));
                }
            }
            KeyCode::Char('x') => {
                // Clear results
                app.runner_result = None;
                app.runner_scroll = 0;
            }
            KeyCode::Esc => {
                app.runner_mode = false;
                app.popup_message = None;
            }
            KeyCode::Char('?') => {
                app.show_help = !app.show_help;
            }
            // Enter is handled in main.rs to start the run
            _ => {}
        }
        return;
    }

    // Handle WebSocket mode inputs
    if app.active_tab().app_mode == crate::app::AppMode::WebSocket {
        match app.active_tab().input_mode {
            InputMode::EditingWsUrl => match key_event.code {
                KeyCode::Enter | KeyCode::Esc => {
                    app.active_tab_mut().input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    app.active_tab_mut().ws_url.push(c);
                }
                KeyCode::Backspace => {
                    app.active_tab_mut().ws_url.pop();
                }
                _ => {}
            },
            InputMode::EditingWsMessage => match key_event.code {
                KeyCode::Esc => {
                    app.active_tab_mut().input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    app.active_tab_mut().ws_message_input.push(c);
                }
                KeyCode::Backspace => {
                    app.active_tab_mut().ws_message_input.pop();
                }
                _ => {}
            },
            InputMode::EditingGrpcService => match key_event.code {
                KeyCode::Enter | KeyCode::Esc => {
                    app.active_tab_mut().input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    app.active_tab_mut().grpc_service.push(c);
                }
                KeyCode::Backspace => {
                    app.active_tab_mut().grpc_service.pop();
                }
                _ => {}
            },
            InputMode::EditingGrpcProto => match key_event.code {
                KeyCode::Enter | KeyCode::Esc => {
                    app.active_tab_mut().input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    app.active_tab_mut().grpc_proto_path.push(c);
                }
                KeyCode::Backspace => {
                    app.active_tab_mut().grpc_proto_path.pop();
                }
                _ => {}
            },
            InputMode::Normal => match key_event.code {
                KeyCode::Char('e') => {
                    app.active_tab_mut().input_mode = InputMode::EditingWsUrl;
                }
                KeyCode::Char('i') => {
                    app.active_tab_mut().input_mode = InputMode::EditingWsMessage;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    let tab = app.active_tab();
                    let len = tab.ws_messages.len();
                    if tab.ws_scroll < len.saturating_sub(1) {
                        app.active_tab_mut().ws_scroll += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.active_tab().ws_scroll > 0 {
                        app.active_tab_mut().ws_scroll -= 1;
                    }
                }
                KeyCode::Char('x') => {
                    // Clear message history
                    let tab = app.active_tab_mut();
                    tab.ws_messages.clear();
                    tab.ws_scroll = 0;
                }
                KeyCode::Char('?') => {
                    app.show_help = !app.show_help;
                }
                KeyCode::Esc => {
                    app.popup_message = None;
                }
                _ => {}
            },
            _ => {}
        }
        return;
    }

    if app.active_sidebar {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => app.next_collection_item(),
            KeyCode::Char('k') | KeyCode::Up => app.previous_collection_item(),
            KeyCode::Enter => {
                app.load_selected_request();
            }
            KeyCode::Char('D') => {
                if let Some(hist_idx) = app.get_selected_history_index() {
                    app.toggle_diff_selection(hist_idx);
                }
            }
            KeyCode::Esc => app.active_sidebar = false,
            _ => {}
        }
        return;
    }

    match app.active_tab().input_mode {
        InputMode::EditingStressVUs | InputMode::EditingStressDuration => {
            if key_event.code == KeyCode::Esc {
                app.active_tab_mut().input_mode = InputMode::Normal;
                app.show_stress_modal = false;
            }
        }
        InputMode::EditingSentinelInterval => {}
        InputMode::EditingGrpcService => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.active_tab_mut().grpc_service.push(c);
            }
            KeyCode::Backspace => {
                app.active_tab_mut().grpc_service.pop();
            }
            _ => {}
        },
        InputMode::EditingGrpcProto => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.active_tab_mut().grpc_proto_path.push(c);
            }
            KeyCode::Backspace => {
                app.active_tab_mut().grpc_proto_path.pop();
            }
            _ => {}
        },

        InputMode::CommandPalette => match key_event.code {
            KeyCode::Esc => {
                app.show_command_palette = false;
                app.command_query.clear();
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Down => {
                app.command_index += 1;
            }
            KeyCode::Up => {
                app.command_index = app.command_index.saturating_sub(1);
            }
            KeyCode::Enter => {
                let commands = crate::app::get_available_commands();
                let filter = app.command_query.to_lowercase();
                let filtered: Vec<&crate::app::CommandAction> = commands
                    .iter()
                    .filter(|c| {
                        c.name.to_lowercase().contains(&filter)
                            || c.desc.to_lowercase().contains(&filter)
                    })
                    .collect();

                if let Some(cmd) = filtered.get(app.command_index) {
                    match cmd.name {
                        "New Tab" => {
                            app.tabs.push(crate::app::RequestTab::new());
                            app.active_tab = app.tabs.len() - 1;
                            app.next_request_id += 1;
                            let id_str = app.next_request_id.to_string();
                            app.active_tab_mut().name = format!("Req {}", id_str);
                        }
                        "Close Tab" => {
                            if app.tabs.len() > 1 {
                                app.tabs.remove(app.active_tab);
                                if app.active_tab >= app.tabs.len() {
                                    app.active_tab = app.tabs.len() - 1;
                                }
                            }
                        }
                        "Next Tab" => {
                            app.active_tab = (app.active_tab + 1) % app.tabs.len();
                        }
                        "Prev Tab" => {
                            if app.active_tab > 0 {
                                app.active_tab -= 1;
                            } else {
                                app.active_tab = app.tabs.len() - 1;
                            }
                        }
                        "Duplicate Tab" => {
                            app.duplicate_tab();
                        }
                        "Clear History" => {
                            app.clear_history();
                        }
                        "Clear Cookies" => {
                            app.clear_cookies();
                        }
                        "Manage Cookies" => {
                            app.show_cookie_modal = true;
                            app.show_command_palette = false;
                            return;
                        }
                        "Save Request" => {
                            // Saving requires another modal usually (input name/collection)
                            // Or just save to current if bound.
                            // Currently 's' triggers saving?
                            // Let's just simulate it? No, saving logic is complex.
                            // Skip complex actions for MVP palette.
                        }
                        "Toggle Sidebar" => {
                            app.active_sidebar = !app.active_sidebar;
                        }
                        "Toggle Zen Mode" => {
                            app.zen_mode = !app.zen_mode;
                        }
                        "Switch Theme" => {
                            app.next_theme();
                        }
                        "Filter Collections" => {
                            app.show_sidebar_filter = true;
                            app.active_tab_mut().input_mode = InputMode::FilteringSidebar;
                            app.show_command_palette = false;
                            return;
                        }
                        "Toggle WebSocket" => {
                            app.active_tab_mut().app_mode =
                                if app.active_tab().app_mode == crate::app::AppMode::WebSocket {
                                    crate::app::AppMode::Http
                                } else {
                                    crate::app::AppMode::WebSocket
                                };
                        }
                        "Help" => {
                            app.show_help = !app.show_help;
                        }
                        "Quit" => {
                            std::process::exit(0);
                        }
                        "Export HTML Docs" => {
                            if let Err(e) =
                                crate::features::doc_gen::save_html_docs(&app.collections)
                            {
                                app.active_tab_mut().response =
                                    Some(format!("Error saving docs: {}", e));
                            } else {
                                app.popup_message =
                                    Some("Documentation saved to API_DOCS.html".to_string());
                            }
                        }
                        _ => {}
                    }
                }
                app.show_command_palette = false;
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.command_query.push(c);
                app.command_index = 0;
            }
            KeyCode::Backspace => {
                app.command_query.pop();
                app.command_index = 0;
            }
            _ => {}
        },

        InputMode::Command => match key_event.code {
            KeyCode::Enter => {
                let cmd = app.command_input.trim().to_string();
                if !cmd.is_empty() {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    match parts[0] {
                        "q" | "quit" => std::process::exit(0),
                        "w" | "save" => app
                            .show_notification("Save not implemented via command yet.".to_string()),
                        "theme" => {
                            if parts.len() > 1 {
                                if parts[1] == "matrix" {
                                    app.theme = crate::app::Theme::matrix();
                                    app.theme_index = 1;
                                } else if parts[1] == "cyberpunk" {
                                    app.theme = crate::app::Theme::cyberpunk();
                                    app.theme_index = 2;
                                } else if parts[1] == "default" {
                                    app.theme = crate::app::Theme::default_theme();
                                    app.theme_index = 0;
                                } else {
                                    app.show_notification("Unknown theme".to_string());
                                }
                            } else {
                                app.next_theme();
                            }
                        }
                        "new" => {
                            app.tabs.push(crate::app::RequestTab::new());
                            app.active_tab = app.tabs.len() - 1;
                            app.next_request_id += 1;
                            app.active_tab_mut().name = format!("Req {}", app.next_request_id);
                        }
                        "close" => {
                            if app.tabs.len() > 1 {
                                app.tabs.remove(app.active_tab);
                                if app.active_tab >= app.tabs.len() {
                                    app.active_tab = app.tabs.len() - 1;
                                }
                            }
                        }
                        "zen" => app.zen_mode = !app.zen_mode,
                        _ => app.show_notification(format!("Unknown command: {}", parts[0])),
                    }
                }
                app.active_tab_mut().input_mode = InputMode::Normal;
                app.command_input.clear();
            }
            KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
                app.command_input.clear();
            }
            KeyCode::Char(c) => {
                app.command_input.push(c);
            }
            KeyCode::Backspace => {
                app.command_input.pop();
            }
            _ => {}
        },
        InputMode::FilteringSidebar => match key_event.code {
            KeyCode::Enter => {
                // Keep filter, exit mode
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                // Clear filter, exit mode
                app.sidebar_filter.clear();
                app.show_sidebar_filter = false;
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.sidebar_filter.push(c);
            }
            KeyCode::Backspace => {
                app.sidebar_filter.pop();
            }
            _ => {}
        },
        InputMode::Normal => match key_event.code {
            KeyCode::Char(':') => {
                app.active_tab_mut().input_mode = InputMode::Command;
                app.command_input.clear();
            }
            KeyCode::Char('e') => {
                let mut handled = false;
                match app.active_tab().selected_tab {
                    0 => {
                        if !app.active_tab().params.is_empty()
                            && app.active_tab().params_list_state.selected().is_some()
                        {
                            app.active_tab_mut().input_mode = InputMode::EditingParamKey;
                            handled = true;
                        }
                    }
                    2 => {
                        if app.active_tab().body_type == crate::app::BodyType::FormData
                            && !app.active_tab().form_data.is_empty()
                            && app.active_tab().form_list_state.selected().is_some()
                        {
                            app.active_tab_mut().input_mode = InputMode::EditingFormKey;
                            handled = true;
                        }
                    }
                    3 => {
                        if app.active_tab().auth_type == crate::app::AuthType::Bearer {
                            app.active_tab_mut().input_mode = InputMode::EditingAuth;
                            handled = true;
                        }
                    }
                    4 => {
                        if !app.active_tab().extract_rules.is_empty()
                            && app.active_tab().extract_list_state.selected().is_some()
                        {
                            app.active_tab_mut().input_mode = InputMode::EditingChainKey;
                            handled = true;
                        }
                    }
                    _ => {}
                }

                if !handled {
                    let len = app.active_tab().url.len();
                    app.active_tab_mut().url_cursor_index = len;
                    app.active_tab_mut().input_mode = InputMode::Editing;
                }
            }
            KeyCode::Char('%') => {
                app.show_stress_modal = true;
                app.active_tab_mut().input_mode = InputMode::EditingStressVUs;
            }
            KeyCode::Char('q') => {}
            KeyCode::Char('u') => {
                // Trigger editing gRPC Service if in correct tab/mode
                if app.active_tab().selected_tab == 2
                    && app.active_tab().body_type == crate::app::BodyType::Grpc
                {
                    app.active_tab_mut().input_mode = InputMode::EditingGrpcService;
                } else if app.active_tab().selected_tab == 3
                    && app.active_tab().auth_type == crate::app::AuthType::Basic
                {
                    // Existing basic auth user edit binding
                    app.active_tab_mut().input_mode = InputMode::EditingBasicAuthUser;
                }
            }
            KeyCode::Char('p') => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    app.show_command_palette = true;
                    app.active_tab_mut().input_mode = InputMode::CommandPalette;
                    app.command_query.clear();
                    app.command_index = 0;
                } else if app.active_tab().selected_tab == 2
                    && app.active_tab().body_type == crate::app::BodyType::Grpc
                {
                    app.active_tab_mut().input_mode = InputMode::EditingGrpcProto;
                } else if app.active_tab().selected_tab == 3
                    && app.active_tab().auth_type == crate::app::AuthType::Basic
                {
                    // Existing basic auth pass edit binding
                    app.active_tab_mut().input_mode = InputMode::EditingBasicAuthPass;
                }
            }
            KeyCode::Enter => {
                let tab = app.active_tab();
                if tab.selected_tab == 3 && tab.auth_type == crate::app::AuthType::OAuth2 {
                    app.active_tab_mut().trigger_oauth_flow = true;
                }
            }
            KeyCode::Tab => {
                let current = app.active_tab().selected_tab;
                app.active_tab_mut().selected_tab = (current + 1) % 5;
            }
            KeyCode::Char('m') => {
                if app.active_tab().selected_tab == 2 {
                    let new_type = match app.active_tab().body_type {
                        crate::app::BodyType::Raw => crate::app::BodyType::FormData,
                        crate::app::BodyType::FormData => crate::app::BodyType::GraphQL,
                        crate::app::BodyType::GraphQL => crate::app::BodyType::Grpc,
                        crate::app::BodyType::Grpc => crate::app::BodyType::Raw,
                    };
                    app.active_tab_mut().body_type = new_type;
                } else {
                    app.cycle_method();
                }
            }

            KeyCode::Char('j') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.show_cookie_modal = !app.show_cookie_modal;
                if app.show_cookie_modal {
                    // Reset selection when opening
                    app.cookie_list_state.select(Some(0));
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                let tab = app.active_tab();
                let selected_tab = tab.selected_tab;

                if selected_tab == 0 {
                    let len = tab.params.len();
                    if len > 0 {
                        let current = tab.params_list_state.selected();
                        let next = match current {
                            Some(i) => {
                                if i >= len - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.active_tab_mut().params_list_state.select(Some(next));
                    }
                } else if selected_tab == 2 && tab.body_type == crate::app::BodyType::FormData {
                    let len = tab.form_data.len();
                    if len > 0 {
                        let current = tab.form_list_state.selected();
                        let next = match current {
                            Some(i) => {
                                if i >= len - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.active_tab_mut().form_list_state.select(Some(next));
                    }
                } else if selected_tab == 4 {
                    let len = tab.extract_rules.len();
                    if len > 0 {
                        let current = tab.extract_list_state.selected();
                        let next = match current {
                            Some(i) => {
                                if i >= len - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.active_tab_mut().extract_list_state.select(Some(next));
                    }
                } else {
                    app.next_item();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let tab = app.active_tab();
                let selected_tab = tab.selected_tab;

                if selected_tab == 0 {
                    let len = tab.params.len();
                    if len > 0 {
                        let current = tab.params_list_state.selected();
                        let prev = match current {
                            Some(i) => {
                                if i == 0 {
                                    len - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.active_tab_mut().params_list_state.select(Some(prev));
                    }
                } else if selected_tab == 2 && tab.body_type == crate::app::BodyType::FormData {
                    let len = tab.form_data.len();
                    if len > 0 {
                        let current = tab.form_list_state.selected();
                        let prev = match current {
                            Some(i) => {
                                if i == 0 {
                                    len - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.active_tab_mut().form_list_state.select(Some(prev));
                    }
                } else if selected_tab == 4 {
                    let len = tab.extract_rules.len();
                    if len > 0 {
                        let current = tab.extract_list_state.selected();
                        let prev = match current {
                            Some(i) => {
                                if i == 0 {
                                    len - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.active_tab_mut().extract_list_state.select(Some(prev));
                    }
                } else {
                    app.previous_item();
                }
            }
            KeyCode::PageDown => {
                app.scroll_page_down();
            }
            KeyCode::PageUp => {
                app.scroll_page_up();
            }
            KeyCode::Char('a') => {
                let (selected_tab, body_type) = {
                    let tab = app.active_tab();
                    (tab.selected_tab, tab.body_type)
                };

                if selected_tab == 0 {
                    app.active_tab_mut()
                        .params
                        .push(("new_key".to_string(), "value".to_string()));
                    let len = app.active_tab().params.len();
                    app.active_tab_mut().params_list_state.select(Some(len - 1));
                    app.sync_params_to_url();
                    app.active_tab_mut().input_mode = InputMode::EditingParamKey;
                } else if selected_tab == 2 && body_type == crate::app::BodyType::FormData {
                    app.active_tab_mut().form_data.push((
                        "key".to_string(),
                        "val".to_string(),
                        false,
                    ));
                    let len = app.active_tab().form_data.len();
                    app.active_tab_mut().form_list_state.select(Some(len - 1));
                    app.active_tab_mut().input_mode = InputMode::EditingFormKey;
                } else if selected_tab == 4 {
                    app.active_tab_mut()
                        .extract_rules
                        .push(("new_var".to_string(), "path".to_string()));
                    let len = app.active_tab().extract_rules.len();
                    app.active_tab_mut()
                        .extract_list_state
                        .select(Some(len - 1));
                    app.active_tab_mut().input_mode = InputMode::EditingChainKey;
                }
            }
            KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.duplicate_tab();
            }
            KeyCode::Char('d') => {
                let (selected_tab, body_type) = {
                    let tab = app.active_tab();
                    (tab.selected_tab, tab.body_type)
                };

                if selected_tab == 0 {
                    let i = app.active_tab().params_list_state.selected();
                    let len = app.active_tab().params.len();
                    if let Some(i) = i
                        && len > 0
                        && i < len
                    {
                        app.active_tab_mut().params.remove(i);
                        app.sync_params_to_url();

                        let new_len = app.active_tab().params.len();
                        if new_len == 0 {
                            app.active_tab_mut().params_list_state.select(None);
                        } else if i >= new_len {
                            app.active_tab_mut()
                                .params_list_state
                                .select(Some(new_len - 1));
                        }
                    }
                } else if selected_tab == 2 && body_type == crate::app::BodyType::FormData {
                    let i = app.active_tab().form_list_state.selected();
                    let len = app.active_tab().form_data.len();
                    if let Some(i) = i
                        && len > 0
                        && i < len
                    {
                        app.active_tab_mut().form_data.remove(i);

                        let new_len = app.active_tab().form_data.len();
                        if new_len == 0 {
                            app.active_tab_mut().form_list_state.select(None);
                        } else if i >= new_len {
                            app.active_tab_mut()
                                .form_list_state
                                .select(Some(new_len - 1));
                        }
                    }
                } else if selected_tab == 4 {
                    let i = app.active_tab().extract_list_state.selected();
                    let len = app.active_tab().extract_rules.len();
                    if let Some(i) = i
                        && len > 0
                        && i < len
                    {
                        app.active_tab_mut().extract_rules.remove(i);

                        let new_len = app.active_tab().extract_rules.len();
                        if new_len == 0 {
                            app.active_tab_mut().extract_list_state.select(None);
                        } else if i >= new_len {
                            app.active_tab_mut()
                                .extract_list_state
                                .select(Some(new_len - 1));
                        }
                    }
                }
            }

            KeyCode::Char('l') | KeyCode::Right => {
                app.set_expanded_current_selection(true);
            }
            KeyCode::Char(' ') => {
                let (selected_tab, body_type, i_form) = {
                    let tab = app.active_tab();
                    (
                        tab.selected_tab,
                        tab.body_type,
                        tab.form_list_state.selected(),
                    )
                };

                if selected_tab == 2 && body_type == crate::app::BodyType::FormData {
                    if let Some(i) = i_form
                        && let Some(row) = app.active_tab_mut().form_data.get_mut(i)
                    {
                        row.2 = !row.2;
                    }
                } else {
                    app.toggle_current_selection();
                }
            }
            KeyCode::Char('b') => {
                let tab = app.active_tab();
                if tab.selected_tab == 2 && tab.body_type == crate::app::BodyType::Raw {
                    app.active_tab_mut().selected_tab = 2;
                    app.trigger_editor();
                } else {
                    app.active_tab_mut().selected_tab = 2;
                }
            }
            KeyCode::Char('?') => {
                app.show_help = !app.show_help;
            }
            KeyCode::Char('z') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.zen_mode = !app.zen_mode;
            }
            KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.add_tab();
            }
            KeyCode::Char('x') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.close_tab();
            }
            KeyCode::Char(']') => {
                app.next_tab();
            }
            KeyCode::Char('[') => {
                app.prev_tab();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.set_expanded_current_selection(false);
            }
            KeyCode::Char('H') => {
                app.active_tab_mut().selected_tab = 1;
                app.trigger_header_editor();
            }
            KeyCode::Char('s') => {
                app.save_current_request();
            }
            KeyCode::Char('D') => {
                app.download_response();
            }
            KeyCode::Char('P') => {
                if app.active_tab().response_is_binary {
                    app.preview_response();
                } else {
                    // Open pre-request script editor
                    app.editor_mode = crate::app::EditorMode::PreRequestScript;
                }
            }
            KeyCode::Char('T') => {
                // Open post-request script editor
                app.editor_mode = crate::app::EditorMode::PostRequestScript;
            }
            KeyCode::Char('c') => {
                let cmd = app.generate_curl_command();
                app.copy_to_clipboard(cmd);
            }
            KeyCode::Char('I') => {
                // Import from cURL command
                app.curl_import_input.clear();
                app.active_tab_mut().input_mode = InputMode::ImportCurl;
            }
            KeyCode::Char('G') => {
                let code = app.generate_python_code();
                app.copy_to_clipboard(code);
                app.show_notification("Copied Python Code".to_string());
            }
            KeyCode::Char('J') => {
                let code = app.generate_javascript_code();
                app.copy_to_clipboard(code);
                app.show_notification("Copied JS Code".to_string());
            }
            KeyCode::Char('C') => {
                // Copy response output to clipboard
                app.copy_response();
            }
            KeyCode::Char('O') => {
                let code = app.generate_go_code();
                app.copy_to_clipboard(code);
                app.show_notification("Copied Go Code".to_string());
            }
            KeyCode::Char('R') => {
                let code = app.generate_rust_code();
                app.copy_to_clipboard(code);
                app.show_notification("Copied Rust Code".to_string());
            }

            KeyCode::Char('B') => {
                let code = app.generate_ruby_code();
                app.copy_to_clipboard(code);
                app.show_notification("Copied Ruby Code".to_string());
            }
            KeyCode::Char('E') => {
                let code = app.generate_php_code();
                app.copy_to_clipboard(code);
                app.show_notification("Copied PHP Code".to_string());
            }
            KeyCode::Char('S') => {
                let code = app.generate_csharp_code();
                app.copy_to_clipboard(code);
                app.show_notification("Copied C# Code".to_string());
            }
            KeyCode::Char('M') => {
                app.generate_docs();
            }
            KeyCode::Char('L') => {
                // Only if in gRPC mode - list services via reflection
                if app.active_tab().body_type == crate::app::BodyType::Grpc {
                    app.active_tab_mut().should_list_grpc_services = true;
                }
            }
            KeyCode::Char('i') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Only if in GraphQL mode
                if app.active_tab().selected_tab == 2
                    && app.active_tab().body_type == crate::app::BodyType::GraphQL
                {
                    app.trigger_introspection();
                } else {
                    app.show_notification(
                        "Introspection only available in GraphQL body mode".to_string(),
                    );
                }
            }
            KeyCode::Esc => {
                app.popup_message = None;
            }
            KeyCode::Char('/') => {
                app.active_tab_mut().input_mode = InputMode::Search;
                app.active_tab_mut().search_query.clear();
            }
            KeyCode::Char('f') => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    app.show_sidebar_filter = true;
                    app.active_tab_mut().input_mode = InputMode::FilteringSidebar;
                } else {
                    app.active_tab_mut().fullscreen_response =
                        !app.active_tab().fullscreen_response;
                }
            }
            KeyCode::Char('y') => {
                let tab = app.active_tab();
                if let Some(selected_idx) = tab.json_list_state.selected()
                    && let Some(entries) = &tab.response_json
                {
                    let filter = &tab.search_query;
                    let path = crate::ui::get_json_path(entries, selected_idx, filter);
                    app.copy_to_clipboard(path);
                }
            }
            KeyCode::Char('Q') => {
                if app.active_tab().selected_tab == 2
                    && app.active_tab().body_type == crate::app::BodyType::GraphQL
                {
                    app.editor_mode = crate::app::EditorMode::GraphQLQuery;
                }
            }
            KeyCode::Char('V') => {
                if app.active_tab().selected_tab == 2
                    && app.active_tab().body_type == crate::app::BodyType::GraphQL
                {
                    app.editor_mode = crate::app::EditorMode::GraphQLVariables;
                }
            }
            KeyCode::Char('t') => {
                if app.active_tab().selected_tab == 3 {
                    let new_auth = match app.active_tab().auth_type {
                        crate::app::AuthType::None => crate::app::AuthType::Bearer,
                        crate::app::AuthType::Bearer => crate::app::AuthType::Basic,
                        crate::app::AuthType::Basic => crate::app::AuthType::OAuth2,
                        crate::app::AuthType::OAuth2 => crate::app::AuthType::None,
                    };
                    app.active_tab_mut().auth_type = new_auth;
                }
            }

            KeyCode::Char('1') => {
                if app.active_tab().selected_tab == 3
                    && app.active_tab().auth_type == crate::app::AuthType::OAuth2
                {
                    app.active_tab_mut().input_mode = InputMode::EditingOAuthUrl;
                }
            }
            KeyCode::Char('2') => {
                if app.active_tab().selected_tab == 3
                    && app.active_tab().auth_type == crate::app::AuthType::OAuth2
                {
                    app.active_tab_mut().input_mode = InputMode::EditingOAuthTokenUrl;
                }
            }
            KeyCode::Char('i') => {
                if app.active_tab().selected_tab == 3
                    && app.active_tab().auth_type == crate::app::AuthType::OAuth2
                {
                    app.active_tab_mut().input_mode = InputMode::EditingOAuthClientId;
                }
            }
            _ => {}
        },
        InputMode::Editing => match key_event.code {
            KeyCode::Enter => {
                app.active_tab_mut().input_mode = InputMode::Normal;
                app.sync_url_to_params();
            }
            KeyCode::Tab => {
                app.cycle_method();
            }
            KeyCode::Left => {
                let current = app.active_tab().url_cursor_index;
                if current > 0 {
                    app.active_tab_mut().url_cursor_index = current - 1;
                }
            }
            KeyCode::Right => {
                let current = app.active_tab().url_cursor_index;
                let len = app.active_tab().url.len();
                if current < len {
                    app.active_tab_mut().url_cursor_index = current + 1;
                }
            }
            KeyCode::Home => {
                app.active_tab_mut().url_cursor_index = 0;
            }
            KeyCode::End => {
                let len = app.active_tab().url.len();
                app.active_tab_mut().url_cursor_index = len;
            }
            KeyCode::Char(c) => {
                let idx = app.active_tab().url_cursor_index;
                app.active_tab_mut().url.insert(idx, c);
                app.active_tab_mut().url_cursor_index += 1;
            }
            KeyCode::Backspace => {
                let idx = app.active_tab().url_cursor_index;
                if idx > 0 {
                    app.active_tab_mut().url.remove(idx - 1);
                    app.active_tab_mut().url_cursor_index -= 1;
                }
            }
            KeyCode::Delete => {
                let idx = app.active_tab().url_cursor_index;
                let len = app.active_tab().url.len();
                if idx < len {
                    app.active_tab_mut().url.remove(idx);
                }
            }
            KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::EditingAuth => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.active_tab_mut().auth_token.push(c);
            }
            KeyCode::Backspace => {
                app.active_tab_mut().auth_token.pop();
            }
            _ => {}
        },
        InputMode::Search => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.active_tab_mut().search_query.push(c);
            }
            KeyCode::Backspace => {
                app.active_tab_mut().search_query.pop();
            }
            _ => {}
        },
        InputMode::EditingParamKey => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => {
                app.active_tab_mut().input_mode = InputMode::EditingParamValue;
            }
            KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                let i = app.active_tab().params_list_state.selected();
                if let Some(i) = i {
                    if let Some((k, _)) = app.active_tab_mut().params.get_mut(i) {
                        k.push(c);
                    }
                    app.sync_params_to_url();
                }
            }
            KeyCode::Backspace => {
                let i = app.active_tab().params_list_state.selected();
                if let Some(i) = i {
                    if let Some((k, _)) = app.active_tab_mut().params.get_mut(i) {
                        k.pop();
                    }
                    app.sync_params_to_url();
                }
            }
            _ => {}
        },
        InputMode::EditingParamValue => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                let i = app.active_tab().params_list_state.selected();
                if let Some(i) = i {
                    if let Some((_, v)) = app.active_tab_mut().params.get_mut(i) {
                        v.push(c);
                    }
                    app.sync_params_to_url();
                }
            }
            KeyCode::Backspace => {
                let i = app.active_tab().params_list_state.selected();
                if let Some(i) = i {
                    if let Some((_, v)) = app.active_tab_mut().params.get_mut(i) {
                        v.pop();
                    }
                    app.sync_params_to_url();
                }
            }
            _ => {}
        },
        InputMode::EditingBasicAuthUser => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                app.active_tab_mut().input_mode = InputMode::EditingBasicAuthPass;
            }
            KeyCode::Char(c) => {
                app.active_tab_mut().basic_auth_user.push(c);
            }
            KeyCode::Backspace => {
                app.active_tab_mut().basic_auth_user.pop();
            }
            _ => {}
        },
        InputMode::EditingBasicAuthPass => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.active_tab_mut().basic_auth_pass.push(c);
            }
            KeyCode::Backspace => {
                app.active_tab_mut().basic_auth_pass.pop();
            }
            _ => {}
        },

        InputMode::EditingOAuthUrl => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Char(c) => app.active_tab_mut().oauth_auth_url.push(c),
            KeyCode::Backspace => {
                app.active_tab_mut().oauth_auth_url.pop();
            }
            _ => {}
        },
        InputMode::EditingOAuthTokenUrl => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Char(c) => app.active_tab_mut().oauth_token_url.push(c),
            KeyCode::Backspace => {
                app.active_tab_mut().oauth_token_url.pop();
            }
            _ => {}
        },
        InputMode::EditingOAuthClientId => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Char(c) => app.active_tab_mut().oauth_client_id.push(c),
            KeyCode::Backspace => {
                app.active_tab_mut().oauth_client_id.pop();
            }
            _ => {}
        },
        InputMode::EditingChainKey => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => {
                app.active_tab_mut().input_mode = InputMode::EditingChainPath
            }
            KeyCode::Esc => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                let i = app.active_tab().extract_list_state.selected();
                if let Some(i) = i
                    && let Some(rule) = app.active_tab_mut().extract_rules.get_mut(i)
                {
                    rule.0.push(c);
                }
            }
            KeyCode::Backspace => {
                let i = app.active_tab().extract_list_state.selected();
                if let Some(i) = i
                    && let Some(rule) = app.active_tab_mut().extract_rules.get_mut(i)
                {
                    rule.0.pop();
                }
            }
            _ => {}
        },
        InputMode::EditingChainPath => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Esc => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                let i = app.active_tab().extract_list_state.selected();
                if let Some(i) = i
                    && let Some(rule) = app.active_tab_mut().extract_rules.get_mut(i)
                {
                    rule.1.push(c);
                }
            }
            KeyCode::Backspace => {
                let i = app.active_tab().extract_list_state.selected();
                if let Some(i) = i
                    && let Some(rule) = app.active_tab_mut().extract_rules.get_mut(i)
                {
                    rule.1.pop();
                }
            }
            _ => {}
        },
        InputMode::EditingFormKey => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => {
                app.active_tab_mut().input_mode = InputMode::EditingFormValue
            }
            KeyCode::Esc => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                let i = app.active_tab().form_list_state.selected();
                if let Some(i) = i
                    && let Some(row) = app.active_tab_mut().form_data.get_mut(i)
                {
                    row.0.push(c);
                }
            }
            KeyCode::Backspace => {
                let i = app.active_tab().form_list_state.selected();
                if let Some(i) = i
                    && let Some(row) = app.active_tab_mut().form_data.get_mut(i)
                {
                    row.0.pop();
                }
            }
            _ => {}
        },
        InputMode::EditingFormValue => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Esc => app.active_tab_mut().input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                let i = app.active_tab().form_list_state.selected();
                if let Some(i) = i
                    && let Some(row) = app.active_tab_mut().form_data.get_mut(i)
                {
                    row.1.push(c);
                }
            }
            KeyCode::Backspace => {
                let i = app.active_tab().form_list_state.selected();
                if let Some(i) = i
                    && let Some(row) = app.active_tab_mut().form_data.get_mut(i)
                {
                    row.1.pop();
                }
            }
            _ => {}
        },
        // WebSocket input modes are handled earlier in this function
        InputMode::EditingWsUrl | InputMode::EditingWsMessage => {}
        InputMode::ImportCurl => match key_event.code {
            KeyCode::Enter => {
                let curl_cmd = app.curl_import_input.clone();
                match app.import_from_curl(&curl_cmd) {
                    Ok(()) => {
                        app.show_notification("cURL command imported successfully!".to_string());
                    }
                    Err(e) => {
                        app.popup_message = Some(format!("Import error: {}", e));
                    }
                }
                app.curl_import_input.clear();
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                app.curl_import_input.clear();
                app.active_tab_mut().input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.curl_import_input.push(c);
            }
            KeyCode::Backspace => {
                app.curl_import_input.pop();
            }
            _ => {}
        },
    }
}

pub fn handle_mouse_event(
    mouse_event: ratatui::crossterm::event::MouseEvent,
    app: &mut crate::app::App,
) {
    use ratatui::crossterm::event::MouseEventKind;

    // Basic mouse support: Panel Focus
    // We roughly know the layout regions:
    // Sidebar: x < 20%
    // Main: x > 20%

    // Since we don't have access to the exact layout rects here easily (they are computed in ui.rs),
    // we can use approximate percentages based on typical terminal width.
    // For a robust implementation, the UI would need to store the last computed layout Rects in the App state.

    // For now, let's implement scrolling for the help menu which we know is a modal
    if app.show_diff_view {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => {
                let i = app.diff_list_state.selected().unwrap_or(0);
                app.diff_list_state.select(Some(i + 1));
            }
            MouseEventKind::ScrollUp => {
                let i = app.diff_list_state.selected().unwrap_or(0);
                if i > 0 {
                    app.diff_list_state.select(Some(i - 1));
                }
            }
            _ => {}
        }
        return;
    }

    if app.show_help {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => {
                app.help_scroll = app.help_scroll.saturating_add(1);
            }
            MouseEventKind::ScrollUp => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            _ => {}
        }
        return;
    }

    // Scroll JSON response
    match mouse_event.kind {
        MouseEventKind::ScrollDown => {
            // Scroll JSON list if present
            // Note: This matches search result items count if flattened, not just response_json len.
            // But getting exact item count here is hard without re-flattening.
            // We can just increment selection safely.

            let current = app.active_tab().json_list_state.selected().unwrap_or(0);
            app.active_tab_mut()
                .json_list_state
                .select(Some(current + 1));
        }
        MouseEventKind::ScrollUp => {
            let current = app.active_tab().json_list_state.selected().unwrap_or(0);
            if current > 0 {
                app.active_tab_mut()
                    .json_list_state
                    .select(Some(current - 1));
            }
        }
        MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left) => {
            let x = mouse_event.column;
            let y = mouse_event.row;

            // Rough hit testing
            // Sidebar is approx 20% width
            let term_width = ratatui::crossterm::terminal::size().unwrap_or((80, 24)).0;
            let sidebar_width = if app.active_sidebar {
                (term_width as f32 * 0.2) as u16
            } else {
                0
            };

            if x < sidebar_width {
                // Clicked sidebar
                // app.active_sidebar = true; // It's already active
                // Select item at row? Hard without knowing list offset
            } else {
                // Clicked main area
                // Check tabs area (top ~3 rows)
                if y < 3 {
                    // Clicked tabs
                    // Calculate tab width
                    let tab_width = (term_width - sidebar_width) / 5; // 5 tabs
                    let relative_x = x - sidebar_width;
                    let tab_idx = (relative_x / tab_width) as usize;
                    if tab_idx < 5 {
                        app.active_tab_mut().selected_tab = tab_idx;
                    }
                }
            }
        }
        _ => {}
    }
}
