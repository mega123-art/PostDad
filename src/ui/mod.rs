use crate::app::{App, InputMode, JsonEntry};
use crate::ui::sentinel::render_sentinel_mode;
pub mod sentinel;
pub mod syntax;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Sparkline, Tabs, Wrap,
    },
};
use ratatui_image::StatefulImage;
use similar::{ChangeTag, TextDiff};

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
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    f.render_widget(logo, chunks[1]);

    let tagline = Paragraph::new(Line::from(vec![
        Span::styled("⚡ ", Style::default().fg(Color::Yellow)),
        Span::styled(
            "The Terminal-First API Client",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        ),
        Span::styled(" ⚡", Style::default().fg(Color::Yellow)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(tagline, chunks[2]);

    // Version & hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("v0.2.0", Style::default().fg(Color::DarkGray)),
        Span::raw(" • "),
        Span::styled(
            "Press any key to continue...",
            Style::default().fg(Color::DarkGray),
        ),
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

fn flatten_tree(
    entries: &[JsonEntry],
    list_items: &mut Vec<ListItem<'static>>,
    filter: &str,
    line_counter: &mut usize,
) {
    for entry in entries {
        let matches = if filter.is_empty() {
            true
        } else {
            entry.key.to_lowercase().contains(&filter.to_lowercase())
        };

        if matches {
            *line_counter += 1;
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

            let display_text = format!(
                "{:>4} {}{} {}: {}",
                *line_counter, indent, icon, entry.key, val_str
            );
            let style = get_style_for_value(&entry.value);
            list_items.push(ListItem::new(display_text).style(style));
        }

        if entry.is_expanded {
            flatten_tree(&entry.children, list_items, filter, line_counter);
        }
    }
}

pub fn get_json_path(entries: &[JsonEntry], target_idx: usize, filter: &str) -> String {
    let mut current_idx = 0;
    find_path_by_index(entries, target_idx, &mut current_idx, filter, String::new())
        .map(|path| format!("$.{}", path))
        .unwrap_or_default()
}

fn find_path_by_index(
    entries: &[JsonEntry],
    target_idx: usize,
    current_idx: &mut usize,
    filter: &str,
    parent_path: String,
) -> Option<String> {
    for entry in entries {
        let matches = if filter.is_empty() {
            true
        } else {
            entry.key.to_lowercase().contains(&filter.to_lowercase())
        };

        if matches {
            if *current_idx == target_idx {
                if parent_path.is_empty() {
                    return Some(entry.key.clone());
                } else if entry.key.starts_with('[') {
                    return Some(format!("{}{}", parent_path, entry.key));
                } else {
                    return Some(format!("{}.{}", parent_path, entry.key));
                }
            }
            *current_idx += 1;
        }

        if entry.is_expanded {
            let my_path = if parent_path.is_empty() {
                entry.key.clone()
            } else if entry.key.starts_with('[') {
                format!("{}{}", parent_path, entry.key)
            } else {
                format!("{}.{}", parent_path, entry.key)
            };

            if let Some(p) =
                find_path_by_index(&entry.children, target_idx, current_idx, filter, my_path)
            {
                return Some(p);
            }
        }
    }
    None
}

pub fn render(f: &mut Frame, app: &mut App) {
    if app.show_diff_view {
        render_diff_view(f, app);
        return;
    }

    // Splash screen
    if app.show_splash {
        render_splash(f);
        return;
    }

    if app.runner_mode {
        render_runner_mode(f, app);
        return;
    }

    if app.mock_mode {
        render_mock_mode(f, app);
        return;
    }

    if app.active_tab().app_mode == crate::app::AppMode::WebSocket {
        render_websocket_mode(f, app);
        return;
    }

    if app.sentinel_mode {
        render_sentinel_mode(f, app);
        return;
    }

    if app.active_tab().fullscreen_response {
        render_response_area(f, app, f.area());
    } else {
        // Main layout with status bar at bottom
        let main_with_status = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(1)])
            .split(f.area());

        let main_area = main_with_status[0];
        let status_bar_area = main_with_status[1];

        // Render status bar
        render_status_bar(f, app, status_bar_area);

        let chunks = if app.zen_mode {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(0), Constraint::Percentage(100)])
                .split(main_area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(main_area)
        };

        if !app.zen_mode {
            let sidebar_constraints = if app.show_sidebar_filter {
                vec![
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(4),
                ]
            } else {
                vec![Constraint::Min(10), Constraint::Length(4)]
            };

            let sidebar_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(sidebar_constraints)
                .split(chunks[0]);

            let mut main_sidebar_area = sidebar_chunks[0];

            // Render Search Bar if active
            if app.show_sidebar_filter {
                main_sidebar_area = sidebar_chunks[1];
                let search_text = format!(" 🔍 {}_", app.sidebar_filter);
                let search_bar = Paragraph::new(search_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Filter Collections ")
                        .border_style(Style::default().fg(app.theme.highlight)),
                );
                f.render_widget(search_bar, sidebar_chunks[0]);
            }

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
            let filter_text = app.sidebar_filter.to_lowercase();

            collection_items.push(ListItem::new(Span::styled(
                "--- Collections ---",
                Style::default().add_modifier(Modifier::BOLD),
            )));

            for col in &app.collections {
                let mut keys: Vec<&String> = col.requests.keys().collect();
                keys.sort();

                // Check visibility based on filter
                let matches_collection = col.name.to_lowercase().contains(&filter_text);
                let matching_requests: Vec<&&String> = keys
                    .iter()
                    .filter(|k| filter_text.is_empty() || k.to_lowercase().contains(&filter_text))
                    .collect();

                if !filter_text.is_empty() && !matches_collection && matching_requests.is_empty() {
                    continue;
                }

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
                        Span::styled(
                            format!("{} ", col.name),
                            Style::default().fg(app.theme.text_secondary),
                        ),
                        Span::styled(
                            format!(" {} ", req.method),
                            Style::default()
                                .bg(badge_color)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        ),
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
                for (i, log) in app.request_history.iter().enumerate() {
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

                    let mut spans = vec![
                        Span::styled(
                            format!(" {} ", log.method),
                            Style::default()
                                .bg(badge_color)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(format!("{} ", log.status), status_style),
                        Span::styled(format!("({}ms) ", log.latency), lat_style),
                        Span::raw(&log.url),
                    ];

                    if let Some(base_idx) = app.diff_base_index
                        && base_idx == i
                    {
                        spans.insert(
                            0,
                            Span::styled(
                                "[BASE] ",
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        );
                    }

                    let content = Line::from(spans);

                    collection_items.push(ListItem::new(content));
                }
            }

            let collection_list = List::new(collection_items)
                .block(sidebar_block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");
            f.render_stateful_widget(
                collection_list,
                main_sidebar_area,
                &mut app.collection_state,
            );

            // Calculate response size for display
            let response_size = app
                .active_tab()
                .response_bytes
                .as_ref()
                .map(|b| {
                    let bytes = b.len();
                    if bytes >= 1_000_000 {
                        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
                    } else if bytes >= 1_000 {
                        format!("{:.1} KB", bytes as f64 / 1_000.0)
                    } else {
                        format!("{} B", bytes)
                    }
                })
                .unwrap_or_else(|| "—".to_string());

            let latency_display = app
                .active_tab()
                .latency
                .map(|ms| format!("{} ms", ms))
                .unwrap_or_else(|| "—".to_string());

            let sparkline_title = format!(" ⚡ {} │ 📦 {} ", latency_display, response_size);

            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .title(sparkline_title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(app.theme.accent)),
                )
                .data(&app.active_tab().latency_history)
                .style(Style::default().fg(app.theme.success));
            f.render_widget(sparkline, sidebar_chunks[1]);
        }

        let url_border_color = match app.active_tab().input_mode {
            InputMode::Editing => app.theme.border_focus,
            InputMode::Search => app.theme.accent,
            _ => app.theme.border,
        };

        let method_color = match app.active_tab().method.as_str() {
            "GET" => Color::Green,
            "POST" => Color::Yellow,
            "PUT" => Color::Blue,
            "DELETE" => Color::Red,
            _ => Color::White,
        };

        let method_text = Span::styled(
            format!(" {} ", app.active_tab().method),
            Style::default()
                .bg(method_color)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        let url_text = Span::styled(
            format!(" {} ", app.active_tab().url),
            Style::default()
                .fg(app.theme.text_primary)
                .add_modifier(Modifier::BOLD),
        );

        let script_indicator = if !app.active_tab().pre_request_script.trim().is_empty() {
            Span::styled(" 📜 ", Style::default().fg(app.theme.highlight))
        } else {
            Span::raw("")
        };

        let url_title = if !app.active_tab().pre_request_script.trim().is_empty() {
            " URL ('e': edit, 'm': method, 'P': script, Enter: fetch) "
        } else {
            " URL (Press 'e' to edit, 'm' to cycle method, 'P' for script, 'Enter' to fetch) "
        };

        let url_bar = Paragraph::new(ratatui::text::Line::from(vec![
            method_text,
            script_indicator,
            url_text,
        ]))
        .block(
            Block::default()
                .title(url_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(url_border_color)),
        );

        let titles = ["Params", "Headers", "Body", "Auth", "Chain"]
            .iter()
            .cloned()
            .map(ratatui::text::Line::from)
            .collect::<Vec<_>>();

        // Build breadcrumb trail
        let tab_names = ["Params", "Headers", "Body", "Auth", "Chain"];
        let current_tab = tab_names.get(app.active_tab().selected_tab).unwrap_or(&"");
        let body_type_str = match app.active_tab().body_type {
            crate::app::BodyType::Raw => "Raw",
            crate::app::BodyType::FormData => "Form",
            crate::app::BodyType::GraphQL => "GraphQL",
            crate::app::BodyType::Grpc => "gRPC",
        };

        let breadcrumb = if app.active_tab().selected_tab == 2 {
            // Body tab - show body type
            format!(" 📍 HTTP › {} › {} ", current_tab, body_type_str)
        } else {
            format!(" 📍 HTTP › {} ", current_tab)
        };

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(breadcrumb))
            .select(app.active_tab().selected_tab)
            .style(Style::default().fg(app.theme.accent))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(app.theme.border),
            );

        let req_titles = app
            .tabs
            .iter()
            .map(|t| Line::from(t.name.clone()))
            .collect::<Vec<_>>();
        let req_tabs_widget = Tabs::new(req_titles)
            .block(Block::default().borders(Borders::ALL).title(" Open Tabs "))
            .select(app.active_tab)
            .style(Style::default().fg(app.theme.text_secondary))
            .highlight_style(
                Style::default()
                    .fg(app.theme.highlight)
                    .add_modifier(Modifier::BOLD),
            );

        let main_constraints = if app.zen_mode {
            vec![
                Constraint::Length(3), // Tabs
                Constraint::Length(3), // URL
                Constraint::Min(10),   // Response
            ]
        } else {
            vec![
                Constraint::Length(3), // Tabs
                Constraint::Length(3), // URL
                Constraint::Length(3), // Config Tabs
                Constraint::Length(8), // Config Area
                Constraint::Min(10),   // Response
            ]
        };

        let right_col = Layout::default()
            .direction(Direction::Vertical)
            .constraints(main_constraints)
            .split(chunks[1]);

        f.render_widget(req_tabs_widget, right_col[0]);
        f.render_widget(url_bar, right_col[1]);

        if app.active_tab().input_mode == InputMode::Editing {
            let script_offset = if !app.active_tab().pre_request_script.trim().is_empty() {
                3
            } else {
                0
            };
            let x = right_col[1].x
                + 1
                + (app.active_tab().method.len() as u16 + 2)
                + script_offset
                + 1
                + app.active_tab().url_cursor_index as u16;
            let y = right_col[1].y + 1;
            f.set_cursor_position((x, y));
        }

        if app.zen_mode {
            render_response_area(f, app, right_col[2]);
        } else {
            f.render_widget(tabs, right_col[2]);

            let config_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border));
            match app.active_tab().selected_tab {
                0 => {
                    let mut param_items = Vec::new();
                    let input_mode;
                    {
                        let tab = app.active_tab();
                        input_mode = tab.input_mode;
                        if tab.params.is_empty() {
                            param_items.push(ListItem::new("No params. Press 'a' to add."));
                        } else {
                            for (i, (k, v)) in tab.params.iter().enumerate() {
                                let content = if Some(i) == tab.params_list_state.selected() {
                                    match tab.input_mode {
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
                    }

                    let title = match input_mode {
                        InputMode::EditingParamKey | InputMode::EditingParamValue => {
                            " Params (Editing...) "
                        }
                        _ => " Params (Press 'e' to Edit, 'a' to Add, 'd' to Delete) ",
                    };

                    let style = match input_mode {
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

                    f.render_stateful_widget(
                        list,
                        right_col[2],
                        &mut app.active_tab_mut().params_list_state,
                    );
                }
                1 => {
                    let headers: Vec<ListItem> = app
                        .active_tab()
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
                    let body_type = app.active_tab().body_type;

                    let type_str = match body_type {
                        crate::app::BodyType::Raw => "Raw (Text/JSON)",
                        crate::app::BodyType::FormData => "Multipart Form",
                        crate::app::BodyType::GraphQL => "GraphQL",
                        crate::app::BodyType::Grpc => "gRPC (Proto)",
                    };
                    let main_title = format!(" Body: {} (Press 'm' to switch) ", type_str);

                    match body_type {
                        crate::app::BodyType::Raw => {
                            let request_body = app.active_tab().request_body.clone();
                            let body_txt = if request_body.is_empty() {
                                "No Body. Press 'b' to open editor.".to_string()
                            } else {
                                request_body
                            };

                            // Try to guess extension for highlighting based on content or headers
                            // For request body, we often default to JSON if it looks like it
                            let ext = if body_txt.trim_start().starts_with('{')
                                || body_txt.trim_start().starts_with('[')
                            {
                                "json"
                            } else if body_txt.trim_start().starts_with('<') {
                                "xml" // or html
                            } else {
                                "txt"
                            };

                            let highlighted = crate::ui::syntax::highlight(&body_txt, ext);

                            f.render_widget(
                                Paragraph::new(highlighted)
                                    .block(config_block.title(main_title))
                                    .wrap(Wrap { trim: true }),
                                right_col[2],
                            );
                        }
                        crate::app::BodyType::FormData => {
                            let mut form_items = Vec::new();
                            let input_mode;
                            {
                                let tab = app.active_tab();
                                input_mode = tab.input_mode;
                                if tab.form_data.is_empty() {
                                    form_items
                                        .push(ListItem::new("No form data. Press 'a' to add."));
                                } else {
                                    for (i, (k, v, is_file)) in tab.form_data.iter().enumerate() {
                                        let content = if Some(i) == tab.form_list_state.selected() {
                                            match tab.input_mode {
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
                            }

                            let title = match input_mode {
                                InputMode::EditingFormKey | InputMode::EditingFormValue => {
                                    " Form Data (Editing...) "
                                }
                                _ => {
                                    " Form Data ('e': Edit, 'a': Add, 'd': Del, 'Space': Toggle File) "
                                }
                            };

                            let style = match input_mode {
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
                            f.render_stateful_widget(
                                list,
                                right_col[3],
                                &mut app.active_tab_mut().form_list_state,
                            );
                        }
                        crate::app::BodyType::GraphQL => {
                            f.render_widget(config_block.clone().title(main_title), right_col[3]);
                            let inner = config_block.inner(right_col[3]);
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([
                                    Constraint::Percentage(60),
                                    Constraint::Percentage(40),
                                ])
                                .split(inner);

                            let (graphql_query, graphql_variables) = {
                                let tab = app.active_tab();
                                (tab.graphql_query.clone(), tab.graphql_variables.clone())
                            };

                            let query_txt = if graphql_query.is_empty() {
                                "No Query. Press 'q' to edit.".to_string()
                            } else {
                                graphql_query
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

                            let vars_txt = if graphql_variables.is_empty() {
                                "No Variables. Press 'v' to edit (JSON).".to_string()
                            } else {
                                graphql_variables
                            };
                            let highlighted_vars = crate::ui::syntax::highlight(&vars_txt, "json");
                            f.render_widget(
                                Paragraph::new(highlighted_vars)
                                    .block(
                                        Block::default()
                                            .borders(Borders::TOP)
                                            .title(" Variables (JSON) - 'v' to edit "),
                                    )
                                    .wrap(Wrap { trim: true }),
                                chunks[1],
                            );
                        }
                        crate::app::BodyType::Grpc => {
                            f.render_widget(config_block.clone().title(main_title), right_col[2]);
                            let inner = config_block.inner(right_col[2]);
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([
                                    Constraint::Length(3), // Service/Method
                                    Constraint::Length(3), // Proto Path
                                    Constraint::Min(0),    // Payload
                                ])
                                .split(inner);

                            let tab = app.active_tab();
                            let service_txt = format!(
                                "Service/Method: {} (Press 'u' to edit)",
                                if tab.grpc_service.is_empty() {
                                    "None"
                                } else {
                                    &tab.grpc_service
                                }
                            );

                            f.render_widget(Paragraph::new(service_txt), chunks[0]);

                            let proto_txt = format!(
                                "Proto Path: {} (Press 'p' to edit)",
                                if tab.grpc_proto_path.is_empty() {
                                    "None"
                                } else {
                                    &tab.grpc_proto_path
                                }
                            );
                            f.render_widget(Paragraph::new(proto_txt), chunks[1]);

                            let body_txt = if tab.request_body.is_empty() {
                                "No Payload. Press 'b' to edit (JSON format).".to_string()
                            } else {
                                tab.request_body.clone()
                            };
                            f.render_widget(
                                Paragraph::new(body_txt).wrap(Wrap { trim: true }),
                                chunks[2],
                            );
                        }
                    }
                }
                3 => {
                    let auth_type = app.active_tab().auth_type;
                    let type_str = match auth_type {
                        crate::app::AuthType::None => "None",
                        crate::app::AuthType::Bearer => "Bearer Token",
                        crate::app::AuthType::Basic => "Basic Auth",
                        crate::app::AuthType::OAuth2 => "OAuth 2.0",
                    };
                    let main_title =
                        format!(" Authentication: {} (Press 't' to switch) ", type_str);

                    match auth_type {
                        crate::app::AuthType::None => {
                            f.render_widget(
                                Paragraph::new("No Authentication selected.")
                                    .block(config_block.title(main_title)),
                                right_col[3],
                            );
                        }
                        crate::app::AuthType::Bearer => {
                            let (input_mode, auth_token) = {
                                let tab = app.active_tab();
                                (tab.input_mode, tab.auth_token.clone())
                            };
                            let title = if input_mode == InputMode::EditingAuth {
                                " Bearer Token (Editing) "
                            } else {
                                " Bearer Token (Press 'e' to Edit) "
                            };
                            let style = if input_mode == InputMode::EditingAuth {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            };
                            let auth_txt = if auth_token.is_empty() {
                                "No token set".to_string()
                            } else {
                                auth_token
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
                                right_col[3],
                            );
                        }
                        crate::app::AuthType::Basic => {
                            let (input_mode, basic_auth_user, basic_auth_pass) = {
                                let tab = app.active_tab();
                                (
                                    tab.input_mode,
                                    tab.basic_auth_user.clone(),
                                    tab.basic_auth_pass.clone(),
                                )
                            };

                            let user_style = if input_mode == InputMode::EditingBasicAuthUser {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            };
                            let pass_style = if input_mode == InputMode::EditingBasicAuthPass {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            };

                            let user_txt = format!("Username: {}", basic_auth_user);
                            let pass_txt =
                                format!("Password: {}", "*".repeat(basic_auth_pass.len()));

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
                                right_col[3],
                            );
                        }
                        crate::app::AuthType::OAuth2 => {
                            let (input_mode, client_id, auth_url, token_url, auth_token) = {
                                let tab = app.active_tab();
                                (
                                    tab.input_mode,
                                    tab.oauth_client_id.clone(),
                                    tab.oauth_auth_url.clone(),
                                    tab.oauth_token_url.clone(),
                                    tab.auth_token.clone(),
                                )
                            };

                            let id_style = if input_mode == InputMode::EditingOAuthClientId {
                                Style::default().fg(app.theme.border_focus)
                            } else {
                                Style::default()
                            };
                            let url1_style = if input_mode == InputMode::EditingOAuthUrl {
                                Style::default().fg(app.theme.border_focus)
                            } else {
                                Style::default()
                            };
                            let url2_style = if input_mode == InputMode::EditingOAuthTokenUrl {
                                Style::default().fg(app.theme.border_focus)
                            } else {
                                Style::default()
                            };

                            let content = vec![
                                ListItem::new(format!("Client ID: {}", client_id)).style(id_style),
                                ListItem::new(format!("Auth URL: {}", auth_url)).style(url1_style),
                                ListItem::new(format!("Token URL: {}", token_url))
                                    .style(url2_style),
                                ListItem::new(Line::from(vec![
                                    Span::raw("Status: "),
                                    if !auth_token.is_empty() {
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
                                right_col[3],
                            );
                        }
                    }
                }
                4 => {
                    let mut extract_items = Vec::new();
                    let input_mode;
                    {
                        let tab = app.active_tab();
                        input_mode = tab.input_mode;
                        if tab.extract_rules.is_empty() {
                            extract_items
                                .push(ListItem::new("No chaining rules. Press 'a' to add."));
                        } else {
                            for (i, (key, path)) in tab.extract_rules.iter().enumerate() {
                                let content = if Some(i) == tab.extract_list_state.selected() {
                                    match tab.input_mode {
                                        InputMode::EditingChainKey => {
                                            format!("{} _ <- {}", key, path)
                                        }
                                        InputMode::EditingChainPath => {
                                            format!("{} <- {} _", key, path)
                                        }
                                        _ => format!("{} <- {}", key, path),
                                    }
                                } else {
                                    format!("{} <- {}", key, path)
                                };
                                extract_items.push(ListItem::new(content));
                            }
                        }
                    }

                    let title = match input_mode {
                        InputMode::EditingChainKey | InputMode::EditingChainPath => {
                            " Post-Request Variables (Editing...) "
                        }
                        _ => {
                            " Post-Request Variables (Press 'e' to Edit, 'a' to Add, 'd' to Delete) "
                        }
                    };

                    let style = match input_mode {
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

                    f.render_stateful_widget(
                        list,
                        right_col[3],
                        &mut app.active_tab_mut().extract_list_state,
                    );
                }
                _ => {}
            };

            render_response_area(f, app, right_col[4]);
        }
    }

    if app.active_tab().show_schema_modal {
        render_schema_modal(f, app);
    }

    if app.active_tab().show_grpc_services_modal {
        render_grpc_services_modal(f, app);
    }

    if app.active_tab().show_grpc_description_modal {
        render_grpc_description_modal(f, app);
    }

    if let Some(msg) = &app.popup_message {
        let area = centered_rect(60, 20, f.area());

        // Clear and Render Popup
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(Span::styled(
                " 🔔 Notification ",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title_bottom(Span::styled(
                " Press Esc to close ",
                Style::default().fg(app.theme.text_secondary),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(app.theme.highlight))
            .style(
                Style::default()
                    .bg(app.theme.background)
                    .fg(app.theme.text_primary),
            );

        let para = Paragraph::new(msg.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(para, area);
    }

    fn render_response_area(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
        let mut main_area = area;

        // Split for Test Results/Console if present
        let (has_tests, has_output, fullscreen) = {
            let tab = app.active_tab();
            (
                !tab.test_results.is_empty(),
                !tab.script_output.is_empty(),
                tab.fullscreen_response,
            )
        };

        if has_tests || has_output {
            let size = if fullscreen {
                ratatui::layout::Constraint::Percentage(30)
            } else {
                ratatui::layout::Constraint::Length(10)
            };

            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([size, ratatui::layout::Constraint::Min(0)])
                .split(area);

            let test_area = chunks[0];
            main_area = chunks[1];

            // Render Tests
            let mut lines = Vec::new();
            {
                let tab = app.active_tab();
                if !tab.test_results.is_empty() {
                    let passed = tab.test_results.iter().filter(|(_, p)| *p).count();
                    let total = tab.test_results.len();
                    let summary_color = if passed == total {
                        app.theme.success
                    } else {
                        app.theme.error
                    };

                    lines.push(Line::from(vec![
                        Span::styled(
                            "Test Results: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{}/{} Passed", passed, total),
                            Style::default().fg(summary_color),
                        ),
                    ]));

                    for (name, passed) in &tab.test_results {
                        let (icon, color) = if *passed {
                            ("✅", app.theme.success)
                        } else {
                            ("❌", app.theme.error)
                        };
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(icon, Style::default().fg(color)),
                            Span::raw(format!(" {} ", name)),
                        ]));
                    }
                    lines.push(Line::from(""));
                }

                if !tab.script_output.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "Console Output:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                    for log in &tab.script_output {
                        lines.push(Line::from(Span::styled(
                            format!("  > {}", log),
                            Style::default().fg(app.theme.text_secondary),
                        )));
                    }
                }
            }

            let para = Paragraph::new(lines).block(
                Block::default()
                    .title(" Tests & Console ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.accent)),
            );

            f.render_widget(para, test_area);
        }

        let (is_loading, status_code, latency, search_query, input_mode) = {
            let tab = app.active_tab();
            (
                tab.is_loading,
                tab.status_code,
                tab.latency,
                tab.search_query.clone(),
                tab.input_mode,
            )
        };

        let status_bar_text = if is_loading {
            let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            format!(" {} Fetching... ", spinner_frames[app.spinner_state % 10])
        } else {
            match (status_code, latency) {
                (Some(code), Some(ms)) => {
                    let status_emoji = if (200..300).contains(&code) {
                        "✓"
                    } else if (400..500).contains(&code) {
                        "⚠"
                    } else if code >= 500 {
                        "✗"
                    } else {
                        "→"
                    };
                    let mut s = format!(" {} {} | {}ms ", status_emoji, code, ms);
                    let tab = app.active_tab(); // Re-borrow to check lens
                    if !tab.test_results.is_empty() {
                        let passed = tab.test_results.iter().filter(|(_, p)| *p).count();
                        s.push_str(&format!("| Tests: {}/{} ", passed, tab.test_results.len()));
                    }
                    if !tab.script_output.is_empty() {
                        s.push_str("| Console: Yes ");
                    }
                    s
                }
                (Some(code), None) => {
                    let status_emoji = if (200..300).contains(&code) {
                        "✓"
                    } else if code >= 400 {
                        "✗"
                    } else {
                        "→"
                    };
                    format!(" {} {} ", status_emoji, code)
                }
                _ => " Response ".to_string(),
            }
        };

        let status_style = if let Some(code) = status_code {
            if (200..300).contains(&code) {
                Style::default().fg(app.theme.success)
            } else if code >= 400 {
                Style::default().fg(app.theme.error)
            } else {
                Style::default().fg(app.theme.highlight)
            }
        } else {
            Style::default().fg(app.theme.border)
        };

        let block_title = if input_mode == InputMode::Search {
            format!("{} [Search: {}] ", status_bar_text, search_query)
        } else if !search_query.is_empty() {
            format!("{} [Filter: {}] ", status_bar_text, search_query)
        } else {
            status_bar_text
        };

        // Determine if we have JSON response
        let has_json = app.active_tab().response_json.is_some();

        if has_json {
            let mut items = Vec::new();
            let mut json_path = String::new();
            {
                let tab = app.active_tab();
                if let Some(tree) = &tab.response_json {
                    let mut counter = 0;
                    flatten_tree(tree, &mut items, &tab.search_query, &mut counter);

                    // Get JSON path for selected item
                    if let Some(selected_idx) = tab.json_list_state.selected() {
                        json_path = get_json_path(tree, selected_idx, &tab.search_query);
                    }
                }
            }

            // Build title with JSON path
            let title_with_path = if json_path.is_empty() {
                block_title
            } else {
                format!("{} │ 📍 {}", block_title, json_path)
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(title_with_path)
                        .borders(Borders::ALL)
                        .border_style(status_style),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, main_area, &mut app.active_tab_mut().json_list_state);
        } else if app.active_tab().response_is_binary {
            let img_opt = app.active_tab().response_image.clone();

            if let Some(img) = img_opt
                && let Some(picker) = &mut app.image_picker
            {
                let mut protocol = picker.new_resize_protocol(img);
                let widget = StatefulImage::new();
                f.render_stateful_widget(widget, main_area, &mut protocol);
                return;
            }

            let size = app
                .active_tab()
                .response_bytes
                .as_ref()
                .map(|b| b.len())
                .unwrap_or(0);
            let content = vec![
                Line::from(vec![
                    Span::styled(
                        "📦 Binary Content Detected ",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::raw(format!("({} bytes)", size)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled(
                        "Shift+D",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Cyan),
                    ),
                    Span::raw(" to Download"),
                ]),
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled(
                        "Shift+P",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Cyan),
                    ),
                    Span::raw(" to Preview (Open in Default Viewer)"),
                ]),
            ];

            let para = Paragraph::new(content)
                .block(
                    Block::default()
                        .title(block_title)
                        .borders(Borders::ALL)
                        .border_style(status_style),
                )
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: false });
            f.render_widget(para, main_area);
        } else {
            let content = app
                .active_tab()
                .response
                .as_deref()
                .unwrap_or("No data yet. Press Enter to send request.")
                .to_string(); // clone string to simplify lifetime

            // Highlight response
            let ext = app.guess_extension().unwrap_or("txt".to_string());
            let highlighted = crate::ui::syntax::highlight(&content, &ext);

            let scroll = app.active_tab().response_scroll;

            let para = Paragraph::new(highlighted)
                .block(
                    Block::default()
                        .title(block_title)
                        .borders(Borders::ALL)
                        .border_style(status_style),
                )
                .wrap(Wrap { trim: false })
                .scroll((scroll.0, 0));
            f.render_widget(para, main_area);
        }
    }

    if app.show_help {
        let area = centered_rect(65, 70, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(" Help (j/k: Scroll, ?: Close) ")
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(app.theme.background)
                    .fg(app.theme.text_primary),
            );

        let help_text = vec![
            "General:",
            "  q          Quit",
            "  ?          Toggle Help",
            "  Ctrl+h     Focus Sidebar / Main",
            "  Ctrl+e     Switch Environment",
            "  Ctrl+t     Cycle Themes",
            "  Ctrl+z     Toggle Zen Mode",
            "  Ctrl+p     Command Palette",
            ":          Command Mode",
            "",
            "Request Tabs:",
            "  Ctrl+n     New Tab",
            "  Ctrl+x     Close Tab",
            "  [ / ]      Cycle Open Tabs",
            "  I          Import cURL Command",
            "",
            "Sidebar / History (Focus with Ctrl+h):",
            "  Enter      Load Request",
            "  D          Diff: Select Base (1st) then Target (2nd)",
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
            "  f          Toggle Fullscreen/Sidebar Filter",
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
            "Code Generators (copy to clipboard):",
            "  c          cURL command",
            "  G / J      Python / JavaScript",
            "  O / R      Go / Rust",
            "  B / E      Ruby / PHP",
            "  S          C#",
            "",
            "Response:",
            "  C          Copy Response Output",
            "  D          Download Response (Binary)",
            "  Shift+D    Force Download",
            "  Shift+P    Preview Response (External)",
            "  y          Copy JSON Path",
            "  /          Search / Filter JSON",
            "  (Images render automatically in supported terminals)",
            "",
            "Scripts & Testing:",
            "  P          Edit Pre-Request Script",
            "  T          Edit Post/Test Script",
            "  %          Stress Test (Shift+5)",
            "  S          Sentinel Mode (Live Monitor)",
            "  M          Generate API Docs (MD + HTML)",
            "",
            "Modes:",
            "  Ctrl+w     Toggle WebSocket Mode",
            "  Ctrl+r     Toggle Collection Runner",
            "  Ctrl+k     Mock Server Manager",
            "  Ctrl+j     Cookie Manager",
            "",
            "gRPC (Body Tab -> 'm' to gRPC mode):",
            "  u          Edit Service/Method",
            "  p          Edit Proto file path",
            "  L          List services (reflection)",
            "  D          Describe service (in modal)",
            "  Enter      Send gRPC request",
        ]
        .join("\n");

        let para = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(Color::White))
            .scroll((app.help_scroll, 0));
        f.render_widget(para, area);
    }
    if app.stress_running {
        render_stress_running_overlay(f, app);
    }
    if app.show_stress_modal {
        render_stress_modal(f, app);
    }
    if app.stress_stats.is_some() {
        render_stress_results(f, app);
    }
    if app.show_command_palette {
        render_command_palette(f, app);
    }
    // Render cURL import modal
    if app.active_tab().input_mode == crate::app::InputMode::ImportCurl {
        render_curl_import_modal(f, app);
    }
    if app.show_cookie_modal {
        render_cookie_modal(f, app);
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
        .style(
            Style::default()
                .fg(app.theme.text_primary)
                .add_modifier(Modifier::BOLD),
        )
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
        result_items.push(ListItem::new(Line::from(vec![Span::styled(
            status_text,
            Style::default()
                .fg(app.theme.text_primary)
                .add_modifier(Modifier::BOLD),
        )])));
        result_items.push(ListItem::new("─".repeat(50)));

        // Individual results
        for run in result.results.iter() {
            let status_icon = if run.passed {
                Span::styled(
                    "✓ ",
                    Style::default()
                        .fg(app.theme.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "✗ ",
                    Style::default()
                        .fg(app.theme.error)
                        .add_modifier(Modifier::BOLD),
                )
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

            let latency_str = run
                .latency_ms
                .map(|l| format!("{}ms", l))
                .unwrap_or_default();

            let expected_str = if !run.passed {
                run.expected_status
                    .map(|e| format!("/Exp:{} ", e))
                    .unwrap_or_default()
            } else {
                String::new()
            };

            let mut line_spans = vec![
                status_icon,
                Span::styled(format!("[{}] ", run.method), method_style),
                Span::styled(
                    format!(
                        "{}{}",
                        status_str,
                        if !expected_str.is_empty() { " " } else { "" }
                    ),
                    status_style,
                ),
                Span::styled(expected_str, Style::default().fg(app.theme.error)),
                Span::styled(
                    format!("({}) ", latency_str),
                    Style::default().fg(app.theme.border),
                ),
                Span::styled(
                    format!("{:<20}", run.name),
                    Style::default().fg(app.theme.text_primary),
                ),
            ];

            if !run.tests.is_empty() {
                let passed = run.tests.iter().filter(|(_, p)| *p).count();
                let total = run.tests.len();
                let color = if passed == total {
                    app.theme.success
                } else {
                    app.theme.error
                };
                line_spans.push(Span::styled(
                    format!("Tests:{}/{} ", passed, total),
                    Style::default().fg(color),
                ));
            }

            line_spans.push(Span::styled(
                format!(" {}", run.url),
                Style::default().fg(app.theme.text_secondary),
            ));

            let line = Line::from(line_spans);

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
        list_state.select(Some(
            app.runner_scroll
                .min(result.results.len().saturating_add(1)),
        ));
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
                    Span::styled(
                        format!(" ({} requests)", count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect();

        if collection_items.is_empty() {
            let empty =
                Paragraph::new("No collections found. Add .hcl files to the collections/ folder.")
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
            .style(
                Style::default()
                    .bg(app.theme.background)
                    .fg(app.theme.text_primary),
            );

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

    // Extract basic state
    let (input_mode, ws_connected, ws_url) = {
        let tab = app.active_tab();
        (tab.input_mode, tab.ws_connected, tab.ws_url.clone())
    };

    // URL bar with connection status
    let url_border_color = match input_mode {
        InputMode::EditingWsUrl => app.theme.border_focus,
        _ => app.theme.border,
    };

    let status_indicator = if ws_connected {
        Span::styled(
            " ● ",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD),
        )
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
        format!(" {} ", ws_url),
        Style::default().fg(app.theme.text_primary),
    );

    let url_bar = Paragraph::new(Line::from(vec![ws_label, status_indicator, url_text])).block(
        Block::default()
            .title(if ws_connected {
                " WebSocket (Enter to Disconnect, Ctrl+W for HTTP) "
            } else {
                " WebSocket (Enter to Connect, 'e' to edit URL, Ctrl+W for HTTP) "
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(url_border_color)),
    );
    f.render_widget(url_bar, chunks[0]);

    if input_mode == InputMode::EditingWsUrl {
        let x = chunks[0].x + 1 + 4 + 3 + 1 + ws_url.len() as u16;
        let y = chunks[0].y + 1;
        f.set_cursor_position((x, y));
    }

    // Messages area
    let msg_items: Vec<ListItem> = app
        .active_tab()
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
                Span::styled(
                    format!(" ({})", time_str),
                    Style::default().fg(app.theme.text_secondary),
                ),
            ]))
        })
        .collect();

    let msg_count = app.active_tab().ws_messages.len();
    let msg_title = format!(" Messages ({}) ", msg_count);
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
    let ws_scroll = app.active_tab().ws_scroll;
    if msg_count > 0 {
        ws_list_state.select(Some(ws_scroll.min(msg_count.saturating_sub(1))));
    }
    f.render_stateful_widget(messages_list, chunks[1], &mut ws_list_state);

    // Input field
    let ws_message_input = app.active_tab().ws_message_input.clone();

    let input_border_color = match input_mode {
        InputMode::EditingWsMessage => Color::Yellow,
        _ => Color::Blue,
    };

    let input_text = if ws_message_input.is_empty() && input_mode != InputMode::EditingWsMessage {
        "Press 'i' to type a message..."
    } else {
        &ws_message_input
    };

    let input_title = if ws_connected {
        if input_mode == InputMode::EditingWsMessage {
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

    if input_mode == InputMode::EditingWsMessage {
        let x = chunks[2].x + 1 + ws_message_input.len() as u16;
        let y = chunks[2].y + 1;
        f.set_cursor_position((x, y));
    }

    // Notification popup (Global)
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

    // Help screen for WebSocket mode (Global)
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

pub fn render_diff_view(f: &mut Frame, app: &mut App) {
    if let (Some(base_idx), Some(target_idx)) = (app.diff_base_index, app.diff_target_index)
        && let (Some(base), Some(target)) = (
            app.request_history.get(base_idx),
            app.request_history.get(target_idx),
        )
    {
        let area = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Title
        let title = format!(
            " Diff: Base ({}) vs Target ({}) - Press 'Esc' to close ",
            base.url, target.url
        );
        let block = Block::default().borders(Borders::ALL).title(title);
        f.render_widget(block, area);

        // Inner chunks for diff content
        let content_area = chunks[1];
        let diff_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_area);

        let old_text = base.body.as_deref().unwrap_or("");
        let new_text = target.body.as_deref().unwrap_or("");

        let diff = TextDiff::from_lines(old_text, new_text);

        let mut left_lines = Vec::new();
        let mut right_lines = Vec::new();

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => {
                    left_lines.push(ListItem::new(Line::from(Span::styled(
                        format!("- {}", change),
                        Style::default().bg(Color::Red).fg(Color::Black),
                    ))));
                    right_lines.push(ListItem::new(Line::from("")));
                }
                ChangeTag::Insert => {
                    left_lines.push(ListItem::new(Line::from("")));
                    right_lines.push(ListItem::new(Line::from(Span::styled(
                        format!("+ {}", change),
                        Style::default().bg(Color::Green).fg(Color::Black),
                    ))));
                }
                ChangeTag::Equal => {
                    left_lines.push(ListItem::new(Line::from(format!("  {}", change))));
                    right_lines.push(ListItem::new(Line::from(format!("  {}", change))));
                }
            }
        }

        // Render left
        // Render left
        f.render_stateful_widget(
            List::new(left_lines)
                .block(Block::default().borders(Borders::RIGHT).title(" Base "))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            diff_chunks[0],
            &mut app.diff_list_state,
        );
        // Render right
        f.render_stateful_widget(
            List::new(right_lines)
                .block(Block::default().title(" Target "))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            diff_chunks[1],
            &mut app.diff_list_state,
        );
    }
}

pub fn render_mock_mode(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title
    let status_text = if app.mock_server_running {
        Span::styled(
            " RUNNING ",
            Style::default().bg(Color::Green).fg(Color::Black),
        )
    } else {
        Span::styled(
            " STOPPED ",
            Style::default().bg(Color::Red).fg(Color::Black),
        )
    };

    let port_text = Span::raw(format!(" on port {} ", app.mock_server_port));

    let title = Line::from(vec![
        Span::styled(
            " Mock Server Manager ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        status_text,
        port_text,
    ]);

    let block = Block::default().borders(Borders::ALL).title(title);
    f.render_widget(block, chunks[0]);

    // Content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    // Sidebar List
    let items: Vec<ListItem> = app
        .mock_routes
        .iter()
        .map(|r| {
            ListItem::new(format!("{} {}", r.method, r.path))
                .style(Style::default().fg(Color::Cyan))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title(" Routes "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, content_chunks[0], &mut app.mock_list_state);

    // Details
    if let Some(selected) = app.mock_list_state.selected() {
        if let Some(route) = app.mock_routes.get(selected) {
            let details = vec![
                Line::from(vec![
                    Span::styled("Method: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&route.method),
                ]),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&route.path),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                    Span::raw(route.status.to_string()),
                ]),
                Line::from(""),
                Line::from(Span::styled("Headers:", Style::default().fg(Color::Yellow))),
            ];

            let mut details_list = details;
            if route.headers.is_empty() {
                details_list.push(Line::from("  (None)"));
            } else {
                for (k, v) in &route.headers {
                    details_list.push(Line::from(format!("  {}: {}", k, v)));
                }
            }

            details_list.push(Line::from(""));
            details_list.push(Line::from(Span::styled(
                "Body:",
                Style::default().fg(Color::Yellow),
            )));

            // Add body lines
            for line in route.body.lines() {
                details_list.push(Line::from(line));
            }

            let details_block = Block::default()
                .borders(Borders::NONE)
                .title(" Route Details ");

            // Use Paragraph for details
            let p = Paragraph::new(details_list)
                .block(details_block)
                .wrap(Wrap { trim: false });
            f.render_widget(p, content_chunks[1]);
        }
    } else {
        f.render_widget(
            Paragraph::new("Select a route to view details").alignment(Alignment::Center),
            content_chunks[1],
        );
    }

    // Help
    let help = Paragraph::new(" 'a': Add | 'd': Delete | 's': Toggle Server | 'Esc': Exit ")
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn render_schema_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .title(" GraphQL Schema Types ")
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.accent));

    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);

    let types: Vec<ListItem> = app
        .active_tab()
        .graphql_schema_types
        .iter()
        .map(|t| {
            ListItem::new(Line::from(vec![
                Span::raw("- "),
                Span::styled(t, Style::default().fg(app.theme.highlight)),
            ]))
        })
        .collect();

    let list = List::new(types).block(Block::default().borders(Borders::NONE));

    f.render_widget(list, inner_area);
}

fn render_grpc_services_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .title(" gRPC Services (via Reflection) ")
        .title_bottom(" j/k: Navigate | Enter: Select | Esc: Close ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(app.theme.accent));

    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);

    let services: Vec<ListItem> = app
        .active_tab()
        .grpc_services
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if Some(i) == app.active_tab().form_list_state.selected() {
                Style::default()
                    .fg(app.theme.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text_primary)
            };
            ListItem::new(Line::from(vec![Span::styled(format!("  {} ", s), style)]))
        })
        .collect();

    let list = List::new(services)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, inner_area, &mut app.active_tab_mut().form_list_state);
}

fn render_grpc_description_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(ratatui::widgets::Clear, area);

    let service_name = app.active_tab().grpc_service_to_describe.clone();
    let title = format!(" gRPC Service: {} ", service_name);

    let block = Block::default()
        .title(title)
        .title_bottom(" Esc: Close | b: Back to Services ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(app.theme.accent));

    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);

    let desc = app.active_tab().grpc_service_description.clone();

    // Syntax highlight the proto description
    let highlighted = crate::ui::syntax::highlight(&desc, "protobuf");

    let paragraph = Paragraph::new(highlighted)
        .wrap(Wrap { trim: false })
        .scroll((0, 0));

    f.render_widget(paragraph, inner_area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tab = app.active_tab();

    if tab.input_mode == InputMode::Command {
        let text = format!(":{}█", app.command_input); // Add cursor block
        let p = Paragraph::new(text).style(
            Style::default()
                .bg(app.theme.background)
                .fg(app.theme.text_primary),
        );
        f.render_widget(p, area);
        return;
    }

    // Current mode indicator
    let mode = match tab.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Editing => "EDIT:URL",
        InputMode::Search => "SEARCH",
        InputMode::EditingAuth
        | InputMode::EditingBasicAuthUser
        | InputMode::EditingBasicAuthPass => "EDIT:AUTH",
        InputMode::EditingGrpcService => "EDIT:gRPC",
        InputMode::EditingGrpcProto => "EDIT:PROTO",
        InputMode::EditingWsUrl | InputMode::EditingWsMessage => "EDIT:WS",
        _ => "EDIT",
    };

    let mode_style = match tab.input_mode {
        InputMode::Normal => Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    };

    // Body type indicator
    let body_type = match tab.body_type {
        crate::app::BodyType::Raw => "RAW",
        crate::app::BodyType::FormData => "FORM",
        crate::app::BodyType::GraphQL => "GQL",
        crate::app::BodyType::Grpc => "gRPC",
    };

    // Connection status for WebSocket
    let ws_status = if tab.app_mode == crate::app::AppMode::WebSocket {
        if tab.ws_connected {
            " 🟢 WS "
        } else {
            " 🔴 WS "
        }
    } else {
        ""
    };

    // Build status line
    let left_side = vec![
        Span::styled(format!(" {} ", mode), mode_style),
        Span::raw(" "),
        Span::styled(
            format!(" {} ", tab.method),
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
        Span::raw(" "),
        Span::styled(
            format!(" {} ", body_type),
            Style::default().fg(app.theme.accent),
        ),
        Span::raw(ws_status),
    ];

    // Keybind hints on right side
    let hints = " ?:Help │ e:URL │ Tab:Sections │ Enter:Send │ q:Quit ";

    let right_side = Span::styled(hints, Style::default().fg(app.theme.text_secondary));

    // Calculate padding
    let left_len: usize = left_side.iter().map(|s| s.content.len()).sum();
    let right_len = hints.len();
    let padding = area.width.saturating_sub((left_len + right_len) as u16);

    let mut spans = left_side;
    spans.push(Span::raw(" ".repeat(padding as usize)));
    spans.push(right_side);

    let status_line =
        Paragraph::new(Line::from(spans)).style(Style::default().bg(app.theme.background));

    f.render_widget(status_line, area);
}

fn render_command_palette(f: &mut Frame, app: &mut App) {
    use crate::app::get_available_commands;

    let area = centered_rect(60, 50, f.area());
    f.render_widget(ratatui::widgets::Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let search_bar = Paragraph::new(app.command_query.clone()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Command Palette ")
            .border_style(Style::default().fg(app.theme.highlight)),
    );
    f.render_widget(search_bar, chunks[0]);

    // Filter commands
    let commands = get_available_commands();
    let filter = app.command_query.to_lowercase();
    let filtered: Vec<&crate::app::CommandAction> = commands
        .iter()
        .filter(|c| {
            c.name.to_lowercase().contains(&filter) || c.desc.to_lowercase().contains(&filter)
        })
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|c| {
            let content = Line::from(vec![
                Span::styled(
                    format!("{:<20}", c.name),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(c.desc),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(app.theme.highlight).fg(Color::Black))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if app.command_index >= filtered.len() && !filtered.is_empty() {
        app.command_index = filtered.len() - 1;
    }
    state.select(Some(app.command_index));

    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_stress_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 40, f.area());
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .title(" Stress Test Configuration ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.highlight));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // VUs
            Constraint::Length(3), // Duration
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Help/Info
        ])
        .split(area);

    f.render_widget(block, area);

    let vus_style = if app.active_tab().input_mode == InputMode::EditingStressVUs {
        Style::default().fg(app.theme.border_focus)
    } else {
        Style::default().fg(app.theme.border)
    };

    let dur_style = if app.active_tab().input_mode == InputMode::EditingStressDuration {
        Style::default().fg(app.theme.border_focus)
    } else {
        Style::default().fg(app.theme.border)
    };

    let vus_input = Paragraph::new(app.stress_vus_input.clone()).block(
        Block::default()
            .title(" Virtual Users (Concurrency) ")
            .borders(Borders::ALL)
            .border_style(vus_style),
    );
    f.render_widget(vus_input, chunks[0]);

    let dur_input = Paragraph::new(app.stress_duration_input.clone()).block(
        Block::default()
            .title(" Duration (Seconds) ")
            .borders(Borders::ALL)
            .border_style(dur_style),
    );
    f.render_widget(dur_input, chunks[1]);

    let help_text = vec![
        Line::from("Press Enter to Start"),
        Line::from("Press Tab to Switch Field"),
        Line::from("Press Esc to Cancel"),
    ];
    let help = Paragraph::new(help_text).alignment(Alignment::Center);
    f.render_widget(help, chunks[3]);
}

fn render_curl_import_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 30, f.area());
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .title(" Import cURL Command ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.highlight));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Help
        ])
        .split(area);

    f.render_widget(block, area);

    let input = Paragraph::new(app.curl_import_input.clone()).block(
        Block::default()
            .title(" Paste cURL Command ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border_focus)),
    );
    f.render_widget(input, chunks[0]);

    let help_text = vec![
        Line::from("Press Enter to Import"),
        Line::from("Press Esc to Cancel"),
    ];
    let help = Paragraph::new(help_text).alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_cookie_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 70, f.area());
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .title(" Manage Cookies ")
        .title_bottom(" d: Delete | Esc: Close ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(app.theme.accent));

    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);
    let cookies = app.get_flattened_cookies();

    let items: Vec<ListItem> = cookies
        .iter()
        .enumerate()
        .map(|(i, (host, val))| {
            let style = if Some(i) == app.cookie_list_state.selected() {
                Style::default()
                    .fg(app.theme.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text_primary)
            };

            // Truncate value if too long
            let display_val = if val.len() > 50 {
                format!("{}...", &val[0..47])
            } else {
                val.clone()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" [{}] ", host), Style::default().fg(Color::Yellow)),
                Span::styled(display_val, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, inner_area, &mut app.cookie_list_state);
}

fn render_stress_running_overlay(f: &mut Frame, app: &mut App) {
    let area = f.area();
    // Bottom right corner
    let width = 40;
    let height = 3;
    let x = area.width.saturating_sub(width + 2);
    let y = area.height.saturating_sub(height + 2); // Above status bar
    let rect = ratatui::layout::Rect {
        x,
        y,
        width,
        height,
    };

    f.render_widget(ratatui::widgets::Clear, rect);

    let (reqs, sex) = app.stress_progress.unwrap_or((0, 0));

    let text = format!(" Stress Test Running: {} reqs | {}s ", reqs, sex);
    let p = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(Alignment::Center);

    f.render_widget(p, rect);
}

fn render_stress_results(f: &mut Frame, app: &mut App) {
    if let Some(stats) = &app.stress_stats {
        let area = centered_rect(60, 60, f.area());
        f.render_widget(ratatui::widgets::Clear, area);

        let block = Block::default()
            .title(" Stress Test Results ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.success));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines = vec![
            Line::from(vec![
                Span::raw("Total Requests: "),
                Span::styled(
                    format!("{}", stats.total_requests),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Successful: "),
                Span::styled(
                    format!("{}", stats.successful_requests),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("Failed: "),
                Span::styled(
                    format!("{}", stats.failed_requests),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(vec![
                Span::raw("Errors: "),
                Span::styled(
                    format!("{}", stats.errors_count),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(vec![
                Span::raw("RPS: "),
                Span::styled(
                    format!("{:.2}", stats.rps),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Latency Distribution (ms):",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(vec![
                Span::raw("  Avg: "),
                Span::raw(format!("{:.2}", stats.avg_latency_ms)),
            ]),
            Line::from(vec![
                Span::raw("  Min: "),
                Span::raw(format!("{}", stats.min_latency_ms)),
            ]),
            Line::from(vec![
                Span::raw("  Max: "),
                Span::raw(format!("{}", stats.max_latency_ms)),
            ]),
            Line::from(vec![
                Span::raw("  P50: "),
                Span::raw(format!("{}", stats.p50_latency_ms)),
            ]),
            Line::from(vec![
                Span::raw("  P90: "),
                Span::raw(format!("{}", stats.p90_latency_ms)),
            ]),
            Line::from(vec![
                Span::raw("  P99: "),
                Span::raw(format!("{}", stats.p99_latency_ms)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Status Codes:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
        ];

        // Add status codes
        let mut codes: Vec<&u16> = stats.status_dist.keys().collect();
        codes.sort();
        let mut status_lines = codes
            .iter()
            .map(|code| {
                let count = stats.status_dist.get(code).unwrap();
                let color = if **code >= 200 && **code < 300 {
                    Color::Green
                } else if **code >= 400 {
                    Color::Red
                } else {
                    Color::Yellow
                };
                Line::from(vec![
                    Span::raw(format!("  {}: ", code)),
                    Span::styled(format!("{}", count), Style::default().fg(color)),
                ])
            })
            .collect::<Vec<_>>();

        let mut all_lines = lines;
        all_lines.append(&mut status_lines);
        all_lines.push(Line::from(""));
        all_lines.push(Line::from("Press Esc to Close"));

        f.render_widget(
            Paragraph::new(all_lines).block(Block::default().borders(Borders::NONE)),
            inner,
        );
    }
}
