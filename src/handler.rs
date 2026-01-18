use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    if app.show_splash {
        app.show_splash = false;
        return;
    }

    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            KeyCode::Char('h') => {
                app.active_sidebar = !app.active_sidebar;
                if app.active_sidebar {
                    if app.collection_state.selected().is_none() {
                        app.collection_state.select(Some(0));
                    }
                }
                return;
            }
            KeyCode::Char('e') => {
                app.next_env();
                return;
            }
            KeyCode::Char('w') => {
                // Toggle between HTTP and WebSocket modes
                app.app_mode = match app.app_mode {
                    crate::app::AppMode::Http => crate::app::AppMode::WebSocket,
                    crate::app::AppMode::WebSocket => crate::app::AppMode::Http,
                };
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
            _ => {}
        }
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
    if app.app_mode == crate::app::AppMode::WebSocket {
        match app.input_mode {
            InputMode::EditingWsUrl => match key_event.code {
                KeyCode::Enter | KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    app.ws_url.push(c);
                }
                KeyCode::Backspace => {
                    app.ws_url.pop();
                }
                _ => {}
            },
            InputMode::EditingWsMessage => match key_event.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    app.ws_message_input.push(c);
                }
                KeyCode::Backspace => {
                    app.ws_message_input.pop();
                }
                _ => {}
            },
            InputMode::Normal => match key_event.code {
                KeyCode::Char('e') => {
                    app.input_mode = InputMode::EditingWsUrl;
                }
                KeyCode::Char('i') => {
                    app.input_mode = InputMode::EditingWsMessage;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if app.ws_scroll < app.ws_messages.len().saturating_sub(1) {
                        app.ws_scroll += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.ws_scroll > 0 {
                        app.ws_scroll -= 1;
                    }
                }
                KeyCode::Char('x') => {
                    // Clear message history
                    app.ws_messages.clear();
                    app.ws_scroll = 0;
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
            KeyCode::Esc => app.active_sidebar = false,
            _ => {}
        }
        return;
    }

    match app.input_mode {
        InputMode::Normal => match key_event.code {
            KeyCode::Char('e') => {
                let mut handled = false;
                match app.selected_tab {
                    0 => {
                        if !app.params.is_empty() && app.params_list_state.selected().is_some() {
                            app.input_mode = InputMode::EditingParamKey;
                            handled = true;
                        }
                    }
                    2 => {
                        if app.body_type == crate::app::BodyType::FormData {
                            if !app.form_data.is_empty() && app.form_list_state.selected().is_some()
                            {
                                app.input_mode = InputMode::EditingFormKey;
                                handled = true;
                            }
                        }
                    }
                    3 => {
                        if app.auth_type == crate::app::AuthType::Bearer {
                            app.input_mode = InputMode::EditingAuth;
                            handled = true;
                        }
                    }
                    4 => {
                        if !app.extract_rules.is_empty()
                            && app.extract_list_state.selected().is_some()
                        {
                            app.input_mode = InputMode::EditingChainKey;
                            handled = true;
                        }
                    }
                    _ => {}
                }

                if !handled {
                    app.input_mode = InputMode::Editing;
                }
            }
            KeyCode::Char('q') => {}
            KeyCode::Enter => {
                if app.selected_tab == 3 && app.auth_type == crate::app::AuthType::OAuth2 {
                    app.trigger_oauth_flow = true;
                }
            }
            KeyCode::Tab => {
                app.selected_tab = (app.selected_tab + 1) % 5;
            }
            KeyCode::Char('m') => {
                if app.selected_tab == 2 {
                    app.body_type = match app.body_type {
                        crate::app::BodyType::Raw => crate::app::BodyType::FormData,
                        crate::app::BodyType::FormData => crate::app::BodyType::GraphQL,
                        crate::app::BodyType::GraphQL => crate::app::BodyType::Raw,
                    };
                } else {
                    app.cycle_method();
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                if app.selected_tab == 0 {
                    if !app.params.is_empty() {
                        let next = match app.params_list_state.selected() {
                            Some(i) => {
                                if i >= app.params.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.params_list_state.select(Some(next));
                    }
                } else if app.selected_tab == 2 && app.body_type == crate::app::BodyType::FormData {
                    if !app.form_data.is_empty() {
                        let next = match app.form_list_state.selected() {
                            Some(i) => {
                                if i >= app.form_data.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.form_list_state.select(Some(next));
                    }
                } else if app.selected_tab == 4 {
                    if !app.extract_rules.is_empty() {
                        let next = match app.extract_list_state.selected() {
                            Some(i) => {
                                if i >= app.extract_rules.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.extract_list_state.select(Some(next));
                    }
                } else {
                    app.next_item();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.selected_tab == 0 {
                    if !app.params.is_empty() {
                        let prev = match app.params_list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.params.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.params_list_state.select(Some(prev));
                    }
                } else if app.selected_tab == 2 && app.body_type == crate::app::BodyType::FormData {
                    if !app.form_data.is_empty() {
                        let prev = match app.form_list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.form_data.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.form_list_state.select(Some(prev));
                    }
                } else if app.selected_tab == 4 {
                    if !app.extract_rules.is_empty() {
                        let prev = match app.extract_list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.extract_rules.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.extract_list_state.select(Some(prev));
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
                if app.selected_tab == 0 {
                    app.params
                        .push(("new_key".to_string(), "value".to_string()));
                    app.params_list_state.select(Some(app.params.len() - 1));
                    app.sync_params_to_url();
                    app.input_mode = InputMode::EditingParamKey;
                } else if app.selected_tab == 2 && app.body_type == crate::app::BodyType::FormData {
                    app.form_data
                        .push(("key".to_string(), "val".to_string(), false));
                    app.form_list_state.select(Some(app.form_data.len() - 1));
                    app.input_mode = InputMode::EditingFormKey;
                } else if app.selected_tab == 4 {
                    app.extract_rules
                        .push(("new_var".to_string(), "path".to_string()));
                    app.extract_list_state
                        .select(Some(app.extract_rules.len() - 1));
                    app.input_mode = InputMode::EditingChainKey;
                }
            }
            KeyCode::Char('d') => {
                if app.selected_tab == 0 {
                    if let Some(i) = app.params_list_state.selected() {
                        if !app.params.is_empty() {
                            app.params.remove(i);
                            app.sync_params_to_url();
                            if app.params.is_empty() {
                                app.params_list_state.select(None);
                            } else if i >= app.params.len() {
                                app.params_list_state.select(Some(app.params.len() - 1));
                            }
                        }
                    }
                } else if app.selected_tab == 2 && app.body_type == crate::app::BodyType::FormData {
                    if let Some(i) = app.form_list_state.selected() {
                        if !app.form_data.is_empty() {
                            app.form_data.remove(i);
                            if app.form_data.is_empty() {
                                app.form_list_state.select(None);
                            } else if i >= app.form_data.len() {
                                app.form_list_state.select(Some(app.form_data.len() - 1));
                            }
                        }
                    }
                } else if app.selected_tab == 4 {
                    if let Some(i) = app.extract_list_state.selected() {
                        if !app.extract_rules.is_empty() {
                            app.extract_rules.remove(i);
                            if app.extract_rules.is_empty() {
                                app.extract_list_state.select(None);
                            } else if i >= app.extract_rules.len() {
                                app.extract_list_state
                                    .select(Some(app.extract_rules.len() - 1));
                            }
                        }
                    }
                }
            }

            KeyCode::Char('l') | KeyCode::Right => {
                app.set_expanded_current_selection(true);
            }
            KeyCode::Char(' ') => {
                if app.selected_tab == 2 && app.body_type == crate::app::BodyType::FormData {
                    if let Some(i) = app.form_list_state.selected() {
                        if let Some(row) = app.form_data.get_mut(i) {
                            row.2 = !row.2;
                        }
                    }
                } else {
                    app.toggle_current_selection();
                }
            }
            KeyCode::Char('b') => {
                if app.selected_tab == 2 && app.body_type == crate::app::BodyType::Raw {
                    app.selected_tab = 2;
                    app.trigger_editor();
                } else {
                    app.selected_tab = 2;
                }
            }
            KeyCode::Char('?') => {
                app.show_help = !app.show_help;
            }
            KeyCode::Char('z') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.zen_mode = !app.zen_mode;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.set_expanded_current_selection(false);
            }
            KeyCode::Char('H') => {
                app.selected_tab = 1;
                app.trigger_header_editor();
            }
            KeyCode::Char('s') => {
                app.save_current_request();
            }
            KeyCode::Char('P') => {
                // Open pre-request script editor
                app.editor_mode = crate::app::EditorMode::PreRequestScript;
            }
            KeyCode::Char('c') => {
                let cmd = app.generate_curl_command();
                app.copy_to_clipboard(cmd);
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
            KeyCode::Esc => {
                app.popup_message = None;
            }
            KeyCode::Char('/') => {
                app.input_mode = InputMode::Search;
                app.search_query.clear();
            }
            KeyCode::Char('f') => {
                app.fullscreen_response = !app.fullscreen_response;
            }
            KeyCode::Char('Q') => {
                if app.selected_tab == 2 && app.body_type == crate::app::BodyType::GraphQL {
                    app.editor_mode = crate::app::EditorMode::GraphQLQuery;
                }
            }
            KeyCode::Char('V') => {
                if app.selected_tab == 2 && app.body_type == crate::app::BodyType::GraphQL {
                    app.editor_mode = crate::app::EditorMode::GraphQLVariables;
                }
            }
            KeyCode::Char('t') => {
                if app.selected_tab == 3 {
                    app.auth_type = match app.auth_type {
                        crate::app::AuthType::None => crate::app::AuthType::Bearer,
                        crate::app::AuthType::Bearer => crate::app::AuthType::Basic,
                        crate::app::AuthType::Basic => crate::app::AuthType::OAuth2,
                        crate::app::AuthType::OAuth2 => crate::app::AuthType::None,
                    };
                }
            }
            KeyCode::Char('u') => {
                if app.selected_tab == 3 && app.auth_type == crate::app::AuthType::Basic {
                    app.input_mode = InputMode::EditingBasicAuthUser;
                }
            }
            KeyCode::Char('p') => {
                if app.selected_tab == 3 && app.auth_type == crate::app::AuthType::Basic {
                    app.input_mode = InputMode::EditingBasicAuthPass;
                }
            }
            KeyCode::Char('1') => {
                if app.selected_tab == 3 && app.auth_type == crate::app::AuthType::OAuth2 {
                    app.input_mode = InputMode::EditingOAuthUrl;
                }
            }
            KeyCode::Char('2') => {
                if app.selected_tab == 3 && app.auth_type == crate::app::AuthType::OAuth2 {
                    app.input_mode = InputMode::EditingOAuthTokenUrl;
                }
            }
            KeyCode::Char('i') => {
                if app.selected_tab == 3 && app.auth_type == crate::app::AuthType::OAuth2 {
                    app.input_mode = InputMode::EditingOAuthClientId;
                }
            }
            _ => {}
        },
        InputMode::Editing => match key_event.code {
            KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
                app.sync_url_to_params();
            }
            KeyCode::Tab => {
                app.cycle_method();
            }
            KeyCode::Char(c) => {
                app.url.push(c);
            }
            KeyCode::Backspace => {
                app.url.pop();
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::EditingAuth => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.auth_token.push(c);
            }
            KeyCode::Backspace => {
                app.auth_token.pop();
            }
            _ => {}
        },
        InputMode::Search => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
            }
            KeyCode::Backspace => {
                app.search_query.pop();
            }
            _ => {}
        },
        InputMode::EditingParamKey => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => {
                app.input_mode = InputMode::EditingParamValue;
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                if let Some(i) = app.params_list_state.selected() {
                    if let Some((k, _)) = app.params.get_mut(i) {
                        k.push(c);
                        app.sync_params_to_url();
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = app.params_list_state.selected() {
                    if let Some((k, _)) = app.params.get_mut(i) {
                        k.pop();
                        app.sync_params_to_url();
                    }
                }
            }
            _ => {}
        },
        InputMode::EditingParamValue => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                if let Some(i) = app.params_list_state.selected() {
                    if let Some((_, v)) = app.params.get_mut(i) {
                        v.push(c);
                        app.sync_params_to_url();
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = app.params_list_state.selected() {
                    if let Some((_, v)) = app.params.get_mut(i) {
                        v.pop();
                        app.sync_params_to_url();
                    }
                }
            }
            _ => {}
        },
        InputMode::EditingBasicAuthUser => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                app.input_mode = InputMode::EditingBasicAuthPass;
            }
            KeyCode::Char(c) => {
                app.basic_auth_user.push(c);
            }
            KeyCode::Backspace => {
                app.basic_auth_user.pop();
            }
            _ => {}
        },
        InputMode::EditingBasicAuthPass => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.basic_auth_pass.push(c);
            }
            KeyCode::Backspace => {
                app.basic_auth_pass.pop();
            }
            _ => {}
        },

        InputMode::EditingOAuthUrl => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Char(c) => app.oauth_auth_url.push(c),
            KeyCode::Backspace => {
                app.oauth_auth_url.pop();
            }
            _ => {}
        },
        InputMode::EditingOAuthTokenUrl => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Char(c) => app.oauth_token_url.push(c),
            KeyCode::Backspace => {
                app.oauth_token_url.pop();
            }
            _ => {}
        },
        InputMode::EditingOAuthClientId => match key_event.code {
            KeyCode::Enter | KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Char(c) => app.oauth_client_id.push(c),
            KeyCode::Backspace => {
                app.oauth_client_id.pop();
            }
            _ => {}
        },
        InputMode::EditingChainKey => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => app.input_mode = InputMode::EditingChainPath,
            KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                if let Some(i) = app.extract_list_state.selected() {
                    if let Some(rule) = app.extract_rules.get_mut(i) {
                        rule.0.push(c);
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = app.extract_list_state.selected() {
                    if let Some(rule) = app.extract_rules.get_mut(i) {
                        rule.0.pop();
                    }
                }
            }
            _ => {}
        },
        InputMode::EditingChainPath => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => app.input_mode = InputMode::Normal,
            KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                if let Some(i) = app.extract_list_state.selected() {
                    if let Some(rule) = app.extract_rules.get_mut(i) {
                        rule.1.push(c);
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = app.extract_list_state.selected() {
                    if let Some(rule) = app.extract_rules.get_mut(i) {
                        rule.1.pop();
                    }
                }
            }
            _ => {}
        },
        InputMode::EditingFormKey => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => app.input_mode = InputMode::EditingFormValue,
            KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                if let Some(i) = app.form_list_state.selected() {
                    if let Some(row) = app.form_data.get_mut(i) {
                        row.0.push(c);
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = app.form_list_state.selected() {
                    if let Some(row) = app.form_data.get_mut(i) {
                        row.0.pop();
                    }
                }
            }
            _ => {}
        },
        InputMode::EditingFormValue => match key_event.code {
            KeyCode::Enter | KeyCode::Tab => app.input_mode = InputMode::Normal,
            KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                if let Some(i) = app.form_list_state.selected() {
                    if let Some(row) = app.form_data.get_mut(i) {
                        row.1.push(c);
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = app.form_list_state.selected() {
                    if let Some(row) = app.form_data.get_mut(i) {
                        row.1.pop();
                    }
                }
            }
            _ => {}
        },
        // WebSocket input modes are handled earlier in this function
        InputMode::EditingWsUrl | InputMode::EditingWsMessage => {}
    }
}
