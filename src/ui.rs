use crate::app::{App, InputMode, JsonEntry};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Sparkline, Tabs, Wrap},
};

/// ASCII art logo for PostDad
pub const LOGO: &str = r#"
██████╗  ██████╗ ███████╗████████╗██████╗  █████╗ ██████╗ 
██╔══██╗██╔═══██╗██╔════╝╚══██╔══╝██╔══██╗██╔══██╗██╔══██╗
██████╔╝██║   ██║███████╗   ██║   ██║  ██║███████║██║  ██║
██╔═══╝ ██║   ██║╚════██║   ██║   ██║  ██║██╔══██║██║  ██║
██║     ╚██████╔╝███████║   ██║   ██████╔╝██║  ██║██████╔╝
╚═╝      ╚═════╝ ╚══════╝   ╚═╝   ╚═════╝ ╚═╝  ╚═╝╚═════╝ 
"#;

pub fn render_splash(f: &mut Frame) {
    let area = f.area();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    let logo_lines: Vec<Line> = LOGO
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[1]);

    let tagline = Paragraph::new(Line::from(vec![
        Span::styled("⚡ ", Style::default().fg(Color::Yellow)),
        Span::styled("The Terminal-First API Client", Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)),
        Span::styled(" ⚡", Style::default().fg(Color::Yellow)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(tagline, chunks[2]);

    // Version & hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("v0.2.0", Style::default().fg(Color::DarkGray)),
        Span::raw(" • "),
        Span::styled("Press any key to continue...", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[3]);
}

fn get_style_for_value(v: &serde_json::Value) -> Style {
    match v {
        serde_json::Value::String(_) => Style::default().fg(Color::Green),
        serde_json::Value::Number(_) => Style::default().fg(Color::Blue),
        serde_json::Value::Bool(_) => Style::default().fg(Color::Yellow),
        serde_json::Value::Null => Style::default().fg(Color::Red),
        _ => Style::default(),
    }
}

fn flatten_tree(entries: &[JsonEntry], list_items: &mut Vec<ListItem<'static>>, filter: &str) {
    for entry in entries {
        let matches = if filter.is_empty() {
            true
        } else {
            entry.key.to_lowercase().contains(&filter.to_lowercase())
        };

        if matches {
            let indent = "  ".repeat(entry.level);
            let icon = if entry.children.is_empty() {
                " "
            } else if entry.is_expanded {
                "▼"
            } else {
                "▶"
            };

            let val_str = match &entry.value {
                serde_json::Value::String(s) => format!("\"{}\"", s),
                v => format!("{}", v),
            };

            let display_text = format!("{}{} {}: {}", indent, icon, entry.key, val_str);

            let item = ListItem::new(display_text).style(get_style_for_value(&entry.value));
            list_items.push(item);
        }

        if entry.is_expanded {
            flatten_tree(&entry.children, list_items, filter);
        }
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
    // Splash screen
    if app.show_splash {
        render_splash(f);
        return;
    }

    if app.runner_mode {
        render_runner_mode(f, app);
        return;
    }

    if app.app_mode == crate::app::AppMode::WebSocket {
        render_websocket_mode(f, app);
        return;
    }

    if app.fullscreen_response {
        render_response_area(f, app, f.area());
    } else {
        let chunks = if app.zen_mode {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(0), Constraint::Percentage(100)])
                .split(f.area())
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(f.area())
        };

        if !app.zen_mode {
            let sidebar_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(4)])
                .split(chunks[0]);

            let sidebar_title = format!(" Postdad (Env: {}) ", app.get_active_env().name);
            let sidebar_block = Block::default()
                .title(sidebar_title)
                .borders(Borders::ALL)
                .border_style(if app.active_sidebar {
                    Style::default().fg(app.theme.border_focus)
                } else {
                    Style::default().fg(app.theme.border)
                });

            let mut collection_items = Vec::new();

            collection_items.push(ListItem::new(Span::styled(
                "--- Collections ---",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for col in &app.collections {
                let mut keys: Vec<&String> = col.requests.keys().collect();
                keys.sort();
                for key in keys {
                    let req = &col.requests[key];
                    let badge_color = match req.method.as_str() {
                        "GET" => app.theme.success,
                        "POST" => app.theme.highlight,
                        "PUT" => app.theme.accent,
                        "DELETE" => app.theme.error,
                        _ => app.theme.text_secondary,
                    };

                    let content = Line::from(vec![
                        Span::styled(format!("{} ", col.name), Style::default().fg(app.theme.text_secondary)),
                        Span::styled(format!(" {} ", req.method), Style::default().bg(badge_color).fg(Color::Black).add_modifier(Modifier::BOLD)),
                        Span::raw(format!(" {}", key)),
                    ]);
                    collection_items.push(ListItem::new(content));
                }
            }
            if !app.request_history.is_empty() {
                collection_items.push(ListItem::new(Span::raw(" ")));
                collection_items.push(ListItem::new(Span::styled(
                    "--- History ---",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                for log in &app.request_history {
                    let status_style = if log.status >= 200 && log.status < 300 {
                        Style::default().fg(app.theme.success)
                    } else if log.status >= 400 {
                        Style::default().fg(app.theme.error)
                    } else {
                        Style::default().fg(app.theme.highlight)
                    };

                    let lat_style = if log.latency < 200 {
                        Style::default().fg(app.theme.success)
                    } else if log.latency > 1000 {
                        Style::default().fg(app.theme.error)
                    } else {
                        Style::default()
                    };

                    let badge_color = match log.method.as_str() {
                        "GET" => app.theme.success,
                        "POST" => app.theme.highlight,
                        "PUT" => app.theme.accent,
                        "DELETE" => app.theme.error,
                        _ => app.theme.text_secondary,
                    };

                    let content = Line::from(vec![
                        Span::styled(
                            format!(" {} ", log.method),
                            Style::default().bg(badge_color).fg(Color::Black).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(format!("{} ", log.status), status_style),
                        Span::styled(format!("({}ms) ", log.latency), lat_style),
                        Span::raw(&log.url),
                    ]);

                    collection_items.push(ListItem::new(content));
                }
            }

            let collection_list = List::new(collection_items)
                .block(sidebar_block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");
            f.render_stateful_widget(
                collection_list,
                sidebar_chunks[0],
                &mut app.collection_state,
            );

            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .title(" Latency Heartbeat ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(app.theme.accent)),
                )
                .data(&app.latency_history)
                .style(Style::default().fg(app.theme.success));
            f.render_widget(sparkline, sidebar_chunks[1]);
        }

        let url_border_color = match app.input_mode {
            InputMode::Editing => app.theme.border_focus,
            InputMode::Search => app.theme.accent,
            _ => app.theme.border,
        };

        let method_color = match app.method.as_str() {
            "GET" => Color::Green,
            "POST" => Color::Yellow,
            "PUT" => Color::Blue,
            "DELETE" => Color::Red,
            _ => Color::White,
        };

        let method_text = Span::styled(
            format!(" {} ", app.method),
            Style::default()
                .bg(method_color)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        let url_text = Span::styled(
            format!(" {} ", app.url),
            Style::default()
                .fg(app.theme.text_primary)
                .add_modifier(Modifier::BOLD),
        );

        let script_indicator = if !app.pre_request_script.trim().is_empty() {
            Span::styled(" 📜 ", Style::default().fg(app.theme.highlight))
        } else {
            Span::raw("")
        };

        let url_title = if !app.pre_request_script.trim().is_empty() {
            " URL ('e': edit, 'm': method, 'P': script, Enter: fetch) "
        } else {
            " URL (Press 'e' to edit, 'm' to cycle method, 'P' for script, 'Enter' to fetch) "
        };

        let url_bar = Paragraph::new(ratatui::text::Line::from(vec![method_text, script_indicator, url_text])).block(
            Block::default()
                .title(url_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(url_border_color)),
        );

        let titles = vec!["Params", "Headers", "Body", "Auth", "Chain"]
            .iter()
            .cloned()
            .map(ratatui::text::Line::from)
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Request Configuration "),
            )
            .select(app.selected_tab)
            .style(Style::default().fg(app.theme.accent))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(app.theme.border),
            );

        let main_constraints = if app.zen_mode {
            vec![Constraint::Length(3), Constraint::Min(10)]
        } else {
            vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(10),
            ]
        };

        let right_col = Layout::default()
            .direction(Direction::Vertical)
            .constraints(main_constraints)
            .split(chunks[1]);

        f.render_widget(url_bar, right_col[0]);

        if app.input_mode == InputMode::Editing {
            let x = right_col[0].x + 1 + (app.method.len() as u16 + 2) + 1 + app.url.len() as u16;
            let y = right_col[0].y + 1;
            f.set_cursor_position((x, y));
        }

        if app.zen_mode {
            render_response_area(f, app, right_col[1]);
        } else {
            f.render_widget(tabs, right_col[1]);

            let config_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border));
            match app.selected_tab {
                0 => {
                    let mut param_items = Vec::new();
                    if app.params.is_empty() {
                        param_items.push(ListItem::new("No params. Press 'a' to add."));
                    } else {
                        for (i, (k, v)) in app.params.iter().enumerate() {
                            let content = if Some(i) == app.params_list_state.selected() {
                                match app.input_mode {
                                    InputMode::EditingParamKey => format!("> {}_ = {}", k, v),
                                    InputMode::EditingParamValue => format!("> {} = {}_", k, v),
                                    _ => format!("{} = {}", k, v),
                                }
                            } else {
                                format!("{} = {}", k, v)
                            };
                            param_items.push(ListItem::new(content));
                        }
                    }

                    let title = match app.input_mode {
                        InputMode::EditingParamKey | InputMode::EditingParamValue => {
                            " Params (Editing...) "
                        }
                        _ => " Params (Press 'e' to Edit, 'a' to Add, 'd' to Delete) ",
                    };

                    let style = match app.input_mode {
                        InputMode::EditingParamKey | InputMode::EditingParamValue => {
                            Style::default().fg(app.theme.border_focus)
                        }
                        _ => Style::default().fg(app.theme.border),
                    };

                    let list = List::new(param_items)
                        .block(
                            Block::default()
                                .title(title)
                                .borders(Borders::ALL)
                                .border_style(style),
                        )
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                        .highlight_symbol("> ");

                    f.render_stateful_widget(list, right_col[2], &mut app.params_list_state);
                }
                1 => {
                    let headers: Vec<ListItem> = app
                        .request_headers
                        .iter()
                        .map(|(k, v)| ListItem::new(format!("{}: {}", k, v)))
                        .collect();
                    f.render_widget(
                        List::new(headers).block(config_block.title(" Headers ")),
                        right_col[2],
                    );
                }
                2 => {
                    let type_str = match app.body_type {
                        crate::app::BodyType::Raw => "Raw (Text/JSON)",
                        crate::app::BodyType::FormData => "Multipart Form",
                        crate::app::BodyType::GraphQL => "GraphQL",
                    };
                    let main_title = format!(" Body: {} (Press 'm' to switch) ", type_str);

                    match app.body_type {
                        crate::app::BodyType::Raw => {
                            let body_txt = if app.request_body.is_empty() {
                                "No Body. Press 'b' to open editor."
                            } else {
                                &app.request_body
                            };
                            f.render_widget(
                                Paragraph::new(body_txt)
                                    .block(config_block.title(main_title))
                                    .wrap(Wrap { trim: true }),
                                right_col[2],
                            );
                        }
                        crate::app::BodyType::FormData => {
                            let mut form_items = Vec::new();
                            if app.form_data.is_empty() {
                                form_items.push(ListItem::new("No form data. Press 'a' to add."));
                            } else {
                                for (i, (k, v, is_file)) in app.form_data.iter().enumerate() {
                                    let content = if Some(i) == app.form_list_state.selected() {
                                        match app.input_mode {
                                            InputMode::EditingFormKey => format!(
                                                "{} _ = {} {}",
                                                k,
                                                v,
                                                if *is_file { "[FILE]" } else { "" }
                                            ),
                                            InputMode::EditingFormValue => format!(
                                                "{} = {} _ {}",
                                                k,
                                                v,
                                                if *is_file { "[FILE]" } else { "" }
                                            ),
                                            _ => format!(
                                                "{} = {} {}",
                                                k,
                                                v,
                                                if *is_file { "[FILE]" } else { "" }
                                            ),
                                        }
                                    } else {
                                        format!(
                                            "{} = {} {}",
                                            k,
                                            v,
                                            if *is_file { "[FILE]" } else { "" }
                                        )
                                    };
                                    form_items.push(ListItem::new(content));
                                }
                            }

                            let title = match app.input_mode {
                                InputMode::EditingFormKey | InputMode::EditingFormValue => {
                                    " Form Data (Editing...) "
                                }
                                _ => {
                                    " Form Data ('e': Edit, 'a': Add, 'd': Del, 'Space': Toggle File) "
                                }
                            };

                            let style = match app.input_mode {
                                InputMode::EditingFormKey | InputMode::EditingFormValue => {
                                    Style::default().fg(Color::Yellow)
                                }
                                _ => Style::default().fg(Color::Blue),
                            };

                            let list = List::new(form_items)
                                .block(
                                    Block::default()
                                        .title(main_title)
                                        .borders(Borders::ALL)
                                        .border_style(style)
                                        .title_bottom(title),
                                )
                                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                                .highlight_symbol("> ");
                            f.render_stateful_widget(list, right_col[2], &mut app.form_list_state);
                        }
                        crate::app::BodyType::GraphQL => {
                            f.render_widget(config_block.clone().title(main_title), right_col[2]);
                            let inner = config_block.inner(right_col[2]);
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([
                                    Constraint::Percentage(60),
                                    Constraint::Percentage(40),
                                ])
                                .split(inner);

                            let query_txt = if app.graphql_query.is_empty() {
                                "No Query. Press 'q' to edit."
                            } else {
                                &app.graphql_query
                            };
                            f.render_widget(
                                Paragraph::new(query_txt)
                                    .block(
                                        Block::default()
                                            .borders(Borders::BOTTOM)
                                            .title(" Query (q) "),
                                    )
                                    .wrap(Wrap { trim: true }),
                                chunks[0],
                            );

                            let vars_txt = if app.graphql_variables.is_empty() {
                                "No Variables. Press 'v' to edit (JSON)."
                            } else {
                                &app.graphql_variables
                            };
                            f.render_widget(
                                Paragraph::new(vars_txt)
                                    .block(
                                        Block::default()
                                            .borders(Borders::TOP)
                                            .title(" Variables (v) "),
                                    )
                                    .wrap(Wrap { trim: true }),
                                chunks[1],
                            );
                        }
                    }
                }
                3 => {
                    let type_str = match app.auth_type {
                        crate::app::AuthType::None => "None",
                        crate::app::AuthType::Bearer => "Bearer Token",
                        crate::app::AuthType::Basic => "Basic Auth",
                        crate::app::AuthType::OAuth2 => "OAuth 2.0",
                    };
                    let main_title =
                        format!(" Authentication: {} (Press 't' to switch) ", type_str);

                    match app.auth_type {
                        crate::app::AuthType::None => {
                            f.render_widget(
                                Paragraph::new("No Authentication selected.")
                                    .block(config_block.title(main_title)),
                                right_col[2],
                            );
                        }
                        crate::app::AuthType::Bearer => {
                            let title = if app.input_mode == InputMode::EditingAuth {
                                " Bearer Token (Editing) "
                            } else {
                                " Bearer Token (Press 'e' to Edit) "
                            };
                            let style = if app.input_mode == InputMode::EditingAuth {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            };
                            let auth_txt = if app.auth_token.is_empty() {
                                "No token set"
                            } else {
                                &app.auth_token
                            };
                            f.render_widget(
                                Paragraph::new(auth_txt)
                                    .block(
                                        config_block
                                            .title(main_title)
                                            .borders(Borders::ALL)
                                            .border_style(style)
                                            .title_bottom(title),
                                    )
                                    .wrap(Wrap { trim: true }),
                                right_col[2],
                            );
                        }
                        crate::app::AuthType::Basic => {
                            let user_style = if app.input_mode == InputMode::EditingBasicAuthUser {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            };
                            let pass_style = if app.input_mode == InputMode::EditingBasicAuthPass {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            };

                            let user_txt = format!("Username: {}", app.basic_auth_user);
                            let pass_txt =
                                format!("Password: {}", "*".repeat(app.basic_auth_pass.len()));

                            let content = vec![
                                ListItem::new(user_txt).style(user_style),
                                ListItem::new(pass_txt).style(pass_style),
                            ];

                            f.render_widget(
                                List::new(content).block(
                                    config_block
                                        .title(main_title)
                                        .title_bottom(" Press 'u' for User, 'p' for Pass "),
                                ),
                                right_col[2],
                            );
                        }
                        crate::app::AuthType::OAuth2 => {
                            let id_style = if app.input_mode == InputMode::EditingOAuthClientId {
                                Style::default().fg(app.theme.border_focus)
                            } else {
                                Style::default()
                            };
                            let url1_style = if app.input_mode == InputMode::EditingOAuthUrl {
                                Style::default().fg(app.theme.border_focus)
                            } else {
                                Style::default()
                            };
                            let url2_style = if app.input_mode == InputMode::EditingOAuthTokenUrl {
                                Style::default().fg(app.theme.border_focus)
                            } else {
                                Style::default()
                            };

                            let content = vec![
                                ListItem::new(format!("Client ID: {}", app.oauth_client_id))
                                    .style(id_style),
                                ListItem::new(format!("Auth URL: {}", app.oauth_auth_url))
                                    .style(url1_style),
                                ListItem::new(format!("Token URL: {}", app.oauth_token_url))
                                    .style(url2_style),
                                ListItem::new(Line::from(vec![
                                    Span::raw("Status: "),
                                    if !app.auth_token.is_empty() {
                                        Span::styled(
                                            "Connected (Token Acquired)",
                                            Style::default().fg(app.theme.success),
                                        )
                                    } else {
                                        Span::styled(
                                            "Not Connected",
                                            Style::default().fg(app.theme.error),
                                        )
                                    },
                                ])),
                            ];
                            f.render_widget(
                                List::new(content).block(
                                    config_block.title(main_title).title_bottom(
                                        " 'i': ID, '1': AuthURL, '2': TokenURL, 'Enter': Connect ",
                                    ),
                                ),
                                right_col[2],
                            );
                        }
                    }
                }
                4 => {
                    let mut extract_items = Vec::new();
                    if app.extract_rules.is_empty() {
                        extract_items.push(ListItem::new("No chaining rules. Press 'a' to add."));
                    } else {
                        for (i, (key, path)) in app.extract_rules.iter().enumerate() {
                            let content = if Some(i) == app.extract_list_state.selected() {
                                match app.input_mode {
                                    InputMode::EditingChainKey => format!("{} _ <- {}", key, path),
                                    InputMode::EditingChainPath => format!("{} <- {} _", key, path),
                                    _ => format!("{} <- {}", key, path),
                                }
                            } else {
                                format!("{} <- {}", key, path)
                            };
                            extract_items.push(ListItem::new(content));
                        }
                    }

                    let title = match app.input_mode {
                        InputMode::EditingChainKey | InputMode::EditingChainPath => {
                            " Post-Request Variables (Editing...) "
                        }
                        _ => {
                            " Post-Request Variables (Press 'e' to Edit, 'a' to Add, 'd' to Delete) "
                        }
                    };

                    let style = match app.input_mode {
                        InputMode::EditingChainKey | InputMode::EditingChainPath => {
                            Style::default().fg(app.theme.border_focus)
                        }
                        _ => Style::default().fg(app.theme.border),
                    };

                    let list = List::new(extract_items)
                        .block(
                            Block::default()
                                .title(title)
                                .borders(Borders::ALL)
                                .border_style(style),
                        )
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                        .highlight_symbol("> ");

                    f.render_stateful_widget(list, right_col[2], &mut app.extract_list_state);
                }
                _ => {}
            };

            render_response_area(f, app, right_col[3]);
        }
    }

    if let Some(msg) = &app.popup_message {
        let area = centered_rect(60, 20, f.area());
        
        // Clear and Render Popup
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(Span::styled(" 🔔 Notification ", Style::default().add_modifier(Modifier::BOLD)))
            .title_bottom(Span::styled(" Press Esc to close ", Style::default().fg(app.theme.text_secondary)))
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(app.theme.highlight))
            .style(Style::default().bg(app.theme.background).fg(app.theme.text_primary));

        let para = Paragraph::new(msg.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(para, area);
    }

    fn render_response_area(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
        let status_bar_text = if app.is_loading {
            let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            format!(" {} Fetching... ", spinner_frames[app.spinner_state % 10])
        } else {
            match (app.status_code, app.latency) {
                (Some(code), Some(ms)) => format!(" Status: {} | Time: {}ms ", code, ms),
                (Some(code), None) => format!(" Status: {} ", code),
                _ => " Response ".to_string(),
            }
        };

        let status_style = if let Some(code) = app.status_code {
            if code >= 200 && code < 300 {
                Style::default().fg(app.theme.success)
            } else if code >= 400 {
                Style::default().fg(app.theme.error)
            } else {
                Style::default().fg(app.theme.highlight)
            }
        } else {
            Style::default().fg(app.theme.border)
        };

        let block_title = if app.input_mode == InputMode::Search {
            format!("{} [Search: {}] ", status_bar_text, app.search_query)
        } else if !app.search_query.is_empty() {
            format!("{} [Filter: {}] ", status_bar_text, app.search_query)
        } else {
            status_bar_text
        };

        if let Some(tree) = &app.response_json {
            let mut items = Vec::new();
            flatten_tree(tree, &mut items, &app.search_query);
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(block_title)
                        .borders(Borders::ALL)
                        .border_style(status_style),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, area, &mut app.json_list_state);
        } else {
            let content = app
                .response
                .as_deref()
                .unwrap_or("No data yet. Press Enter to send request.");
            let para = Paragraph::new(content)
                .block(
                    Block::default()
                        .title(block_title)
                        .borders(Borders::ALL)
                        .border_style(status_style),
                )
                .wrap(Wrap { trim: true })
                .scroll(app.response_scroll);
            f.render_widget(para, area);
        }
    }

    if app.show_help {
        let area = centered_rect(60, 60, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(" Help (Press '?' to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(app.theme.background).fg(app.theme.text_primary));

        let help_text = vec![
            "General:",
            "  q          Quit",
            "  ?          Toggle Help",
            "  Ctrl+h     Focus Sidebar / Main",
            "  Ctrl+e     Switch Environment",
            "  Ctrl+t     Cycle Themes",
            "  Ctrl+z     Toggle Zen Mode",
            "",
            "Navigation:",
            "  j / k      Move Up / Down",
            "  h / l      Collapse / Expand JSON",
            "  Space      Toggle JSON / Form File",
            "  Tab        Cycle Tabs (Params, Headers, ...)",
            "",
            "Request:",
            "  e          Edit URL (Tab to Cycle Method)",
            "  m          Cycle Method (GET, POST, ...)",
            "  b          Edit Body (Ext. Editor)",
            "  Q / V      Edit GraphQL Query / Vars",
            "  H          Edit Headers (Ext. Editor)",
            "  f          Toggle Fullscreen",
            "  s          Save Request",
            "  Enter      Send Request",
            "",
            "Params / Chain Tabs:",
            "  a          Add Item",
            "  d          Delete Item",
            "  e          Edit Item",
            "",
            "Auth Tab:",
            "  t          Switch Auth Type",
            "  u / p      Edit User / Pass (Basic)",
            "  i / 1 / 2  Edit OAuth ID / URLs",
            "",
            "Tools:",
            "  /          Search / Filter JSON",
            "  c          Copy as Curl",
            "  G          Copy as Python code",
            "  J          Copy as JavaScript code",
            "  P          Edit Pre-Request Script",
            "  Ctrl+w     Toggle WebSocket Mode",
            "  Ctrl+r     Toggle Collection Runner",
        ]
        .join("\n");

        let para = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(Color::White));
        f.render_widget(para, area);
    }
}

fn render_runner_mode(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(10),   // Main content (collections or results)
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Title bar
    let title = Paragraph::new(" 🏃 Collection Runner ")
        .style(Style::default().fg(app.theme.text_primary).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.accent))
                .title(" Ctrl+R to exit | Enter to run | ?: Help "),
        );
    f.render_widget(title, chunks[0]);

    // Main content: either collection list or results
    if let Some(ref result) = app.runner_result {
        // Show results
        let mut result_items: Vec<ListItem> = Vec::new();

        // Summary header
        let status_text = if result.running {
            format!(
                "🔄 Running... ({}/{}) ",
                result.current_index + 1,
                result.total
            )
        } else {
            format!(
                "✅ {} Passed  ❌ {} Failed  (of {})",
                result.passed, result.failed, result.total
            )
        };
        result_items.push(ListItem::new(Line::from(vec![
            Span::styled(status_text, Style::default().fg(app.theme.text_primary).add_modifier(Modifier::BOLD)),
        ])));
        result_items.push(ListItem::new("─".repeat(50)));

        // Individual results
        for (_i, run) in result.results.iter().enumerate() {
            let status_icon = if run.passed {
                Span::styled("✓ ", Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("✗ ", Style::default().fg(app.theme.error).add_modifier(Modifier::BOLD))
            };

            let method_style = match run.method.as_str() {
                "GET" => Style::default().fg(app.theme.success),
                "POST" => Style::default().fg(app.theme.highlight),
                "PUT" => Style::default().fg(app.theme.accent),
                "DELETE" => Style::default().fg(app.theme.error),
                _ => Style::default(),
            };

            let status_str = match run.status {
                Some(s) => format!("{}", s),
                None => "ERR".to_string(),
            };
            let status_style = if run.passed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let latency_str = run.latency_ms.map(|l| format!("{}ms", l)).unwrap_or_default();

            let line = Line::from(vec![
                status_icon,
                Span::styled(format!("[{}] ", run.method), method_style),
                Span::styled(format!("{} ", status_str), status_style),
                Span::styled(format!("({}) ", latency_str), Style::default().fg(Color::DarkGray)),
                Span::raw(&run.name),
            ]);

            result_items.push(ListItem::new(line));

            // Show error if present
            if let Some(ref err) = run.error {
                result_items.push(ListItem::new(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(format!("Error: {}", err), Style::default().fg(Color::Red)),
                ])));
            }
        }

        let results_list = List::new(result_items)
            .block(
                Block::default()
                    .title(format!(" Results: {} ", result.collection_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title_bottom(" j/k: Scroll | x: Clear | Esc: Exit "),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.runner_scroll.min(result.results.len().saturating_add(1))));
        f.render_stateful_widget(results_list, chunks[1], &mut list_state);
    } else {
        // Show collection list
        let collection_items: Vec<ListItem> = app
            .collections
            .iter()
            .map(|c| {
                let count = c.requests.len();
                ListItem::new(Line::from(vec![
                    Span::styled("📁 ", Style::default().fg(Color::Yellow)),
                    Span::raw(&c.name),
                    Span::styled(format!(" ({} requests)", count), Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect();

        if collection_items.is_empty() {
            let empty = Paragraph::new("No collections found. Add .hcl files to the collections/ folder.")
                .block(
                    Block::default()
                        .title(" Collections ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                );
            f.render_widget(empty, chunks[1]);
        } else {
            let collections_list = List::new(collection_items)
                .block(
                    Block::default()
                        .title(" Select Collection to Run ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue))
                        .title_bottom(" j/k: Navigate | Enter: Run | Esc: Exit "),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("▶ ");

            f.render_stateful_widget(collections_list, chunks[1], &mut app.collection_state);
        }
    }

    // Status bar
    let env_name = if !app.environments.is_empty() {
        app.environments[app.selected_env_index].name.clone()
    } else {
        "No Environment".to_string()
    };

    let status = Paragraph::new(format!(" Environment: {} ", env_name))
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(status, chunks[2]);

    // Notification popup
    if let Some(msg) = &app.popup_message {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(" Notification (Esc to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue).fg(Color::White));
        let para = Paragraph::new(msg.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, area);
    }

    // Help screen for Runner mode
    if app.show_help {
        let area = centered_rect(60, 50, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(" Collection Runner Help (Press '?' to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(app.theme.background).fg(app.theme.text_primary));

        let help_text = vec![
            "Collection Runner Mode:",
            "  Ctrl+r     Exit Runner Mode",
            "  Esc        Exit Runner Mode",
            "  q          Quit Application",
            "  ?          Toggle Help",
            "",
            "Before Run:",
            "  j / k      Navigate collections",
            "  Enter      Run selected collection",
            "",
            "After Run:",
            "  j / k      Scroll through results",
            "  x          Clear results",
            "",
            "Status Code Assertions:",
            "  By default, expects HTTP 200",
            "  Add 'expected_status = XXX' in .hcl",
            "  to specify expected status code",
        ]
        .join("\n");

        let para = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(app.theme.text_primary));
        f.render_widget(para, area);
    }
}

fn render_websocket_mode(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // URL bar
            Constraint::Min(10),   // Messages
            Constraint::Length(3), // Input field
        ])
        .split(f.area());

    // URL bar with connection status
    let url_border_color = match app.input_mode {
        InputMode::EditingWsUrl => app.theme.border_focus,
        _ => app.theme.border,
    };

    let status_indicator = if app.ws_connected {
        Span::styled(" ● ", Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" ○ ", Style::default().fg(app.theme.error))
    };

    let ws_label = Span::styled(
        " WS ",
        Style::default()
            .bg(app.theme.accent)
            .fg(app.theme.background)
            .add_modifier(Modifier::BOLD),
    );

    let url_text = Span::styled(
        format!(" {} ", app.ws_url),
        Style::default().fg(app.theme.text_primary),
    );

    let url_bar = Paragraph::new(Line::from(vec![ws_label, status_indicator, url_text])).block(
        Block::default()
            .title(if app.ws_connected {
                " WebSocket (Enter to Disconnect, Ctrl+W for HTTP) "
            } else {
                " WebSocket (Enter to Connect, 'e' to edit URL, Ctrl+W for HTTP) "
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(url_border_color)),
    );
    f.render_widget(url_bar, chunks[0]);

    if app.input_mode == InputMode::EditingWsUrl {
        let x = chunks[0].x + 1 + 4 + 3 + 1 + app.ws_url.len() as u16;
        let y = chunks[0].y + 1;
        f.set_cursor_position((x, y));
    }

    // Messages area
    let msg_items: Vec<ListItem> = app
        .ws_messages
        .iter()
        .map(|msg| {
            let prefix = if msg.is_sent { "→ " } else { "← " };
            let style = if msg.is_sent {
                Style::default().fg(app.theme.accent)
            } else {
                Style::default().fg(app.theme.success)
            };
            let elapsed = msg.timestamp.elapsed().as_secs();
            let time_str = if elapsed < 60 {
                format!("{}s ago", elapsed)
            } else {
                format!("{}m ago", elapsed / 60)
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::styled(msg.content.clone(), style),
                Span::styled(format!(" ({})", time_str), Style::default().fg(app.theme.text_secondary)),
            ]))
        })
        .collect();

    let msg_title = format!(" Messages ({}) ", app.ws_messages.len());
    let msg_block = Block::default()
        .title(msg_title)
        .title_bottom(" j/k: Scroll | x: Clear | ?: Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border));

    let messages_list = List::new(msg_items)
        .block(msg_block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    // Create a temporary list state for rendering
    let mut ws_list_state = ratatui::widgets::ListState::default();
    if !app.ws_messages.is_empty() {
        ws_list_state.select(Some(app.ws_scroll.min(app.ws_messages.len().saturating_sub(1))));
    }
    f.render_stateful_widget(messages_list, chunks[1], &mut ws_list_state);

    // Input field
    let input_border_color = match app.input_mode {
        InputMode::EditingWsMessage => Color::Yellow,
        _ => Color::Blue,
    };

    let input_text = if app.ws_message_input.is_empty() && app.input_mode != InputMode::EditingWsMessage {
        "Press 'i' to type a message..."
    } else {
        &app.ws_message_input
    };

    let input_title = if app.ws_connected {
        if app.input_mode == InputMode::EditingWsMessage {
            " Message (Enter to Send, Esc to cancel) "
        } else {
            " Message (Press 'i' to type) "
        }
    } else {
        " Connect first to send messages "
    };

    let input_bar = Paragraph::new(input_text).block(
        Block::default()
            .title(input_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(input_border_color)),
    );
    f.render_widget(input_bar, chunks[2]);

    if app.input_mode == InputMode::EditingWsMessage {
        let x = chunks[2].x + 1 + app.ws_message_input.len() as u16;
        let y = chunks[2].y + 1;
        f.set_cursor_position((x, y));
    }

    // Notification popup
    if let Some(msg) = &app.popup_message {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(" Notification (Esc to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue).fg(Color::White));
        let para = Paragraph::new(msg.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, area);
    }

    // Help screen for WebSocket mode
    if app.show_help {
        let area = centered_rect(60, 50, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(" WebSocket Help (Press '?' to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        let help_text = vec![
            "WebSocket Mode:",
            "  Ctrl+w     Switch to HTTP mode",
            "  q          Quit",
            "  ?          Toggle Help",
            "",
            "Connection:",
            "  e          Edit WebSocket URL",
            "  Enter      Connect / Disconnect",
            "",
            "Messaging:",
            "  i          Start typing message",
            "  Enter      Send message (while typing)",
            "  Esc        Cancel typing",
            "",
            "Navigation:",
            "  j / k      Scroll messages Up / Down",
            "  x          Clear message history",
        ]
        .join("\n");

        let para = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(Color::White));
        f.render_widget(para, area);
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
