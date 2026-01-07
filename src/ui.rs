use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap, ListItem, List, Sparkline},
    text::Span,
    Frame,
};
use crate::app::{App, InputMode, JsonEntry};

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
            .constraints([
                Constraint::Min(10),    
                Constraint::Length(4),  
            ])
            .split(chunks[0]);

        let sidebar_title = format!(" Postdad (Env: {}) ", app.get_active_env().name);
        let sidebar_block = Block::default()
            .title(sidebar_title)
            .borders(Borders::ALL)
            .border_style(if app.active_sidebar { 
                Style::default().fg(Color::Yellow) 
            } else { 
                Style::default().fg(Color::Blue) 
            });

        let mut collection_items = Vec::new();
        
        collection_items.push(ListItem::new(Span::styled("--- Collections ---", Style::default().add_modifier(Modifier::BOLD))));
        for col in &app.collections {
            
            let mut keys: Vec<&String> = col.requests.keys().collect();
            keys.sort();
            for key in keys {
                 let req = &col.requests[key];
                 let item = ListItem::new(format!("{} [{}] {}", col.name, req.method, key));
                 collection_items.push(item);
            }
        }
        if !app.request_history.is_empty() {
            collection_items.push(ListItem::new(Span::raw(" ")));
            collection_items.push(ListItem::new(Span::styled("--- History ---", Style::default().add_modifier(Modifier::BOLD))));
            for log in &app.request_history {
                collection_items.push(ListItem::new(log.clone()));
            }
        }
    
        let collection_list = List::new(collection_items)
            .block(sidebar_block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");
        f.render_stateful_widget(collection_list, sidebar_chunks[0], &mut app.collection_state);

        
        
        
        let sparkline = Sparkline::default()
            .block(Block::default().title(" Latency Heartbeat ").borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)))
            .data(&app.latency_history)
            .style(Style::default().fg(Color::Green));
        f.render_widget(sparkline, sidebar_chunks[1]);
    }

    
    let url_border_color = match app.input_mode {
        InputMode::Editing => Color::Yellow,
        InputMode::Search => Color::Magenta,
        _ => Color::Blue, 
    };
    
    let method_color = match app.method.as_str() {
        "GET" => Color::Green, "POST" => Color::Yellow, "PUT" => Color::Blue, "DELETE" => Color::Red, _ => Color::White,
    };

    let method_text = Span::styled(format!(" {} ", app.method), Style::default().bg(method_color).fg(Color::Black).add_modifier(Modifier::BOLD));
    let url_text = Span::styled(format!(" {} ", app.url), Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    
    let url_bar = Paragraph::new(ratatui::text::Line::from(vec![method_text, url_text]))
        .block(Block::default()
            .title(" URL (Press 'e' to edit, 'm' to cycle method, 'Enter' to fetch) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(url_border_color)));
    
    
    let titles = vec![
        " [1] Params ", " [2] Headers ", " [3] Body (b) ", " [4] Auth "
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app.selected_tab)
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    
    let main_constraints = if app.zen_mode {
         vec![
            Constraint::Length(3), 
            Constraint::Min(10),   
         ]
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
        let x = right_col[0].x 
            + 1 // Border
            + (app.method.len() as u16 + 2) // Method padding
            + 1 // URL padding
            + app.url.len() as u16;
        let y = right_col[0].y + 1;
        f.set_cursor_position((x, y));
    }

    if app.zen_mode {
         
         render_response_area(f, app, right_col[1]);
    } else {
         
         
         f.render_widget(tabs, right_col[1]);
         
         
         
         
         let config_block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue));
         match app.selected_tab {
            0 => { 
                 let mut param_items = Vec::new();
                 if let Ok(u) = reqwest::Url::parse(&app.url) {
                     for (k, v) in u.query_pairs() {
                         param_items.push(ListItem::new(format!("{} = {}", k, v)));
                     }
                 }
                 if param_items.is_empty() { param_items.push(ListItem::new("No params (add ?key=val to URL)")); }
                 f.render_widget(List::new(param_items).block(config_block.title(" Params (Read-Only) ")), right_col[2]);
            },
            1 => { 
                let headers: Vec<ListItem> = app.request_headers.iter().map(|(k,v)| ListItem::new(format!("{}: {}", k, v))).collect();
                f.render_widget(List::new(headers).block(config_block.title(" Headers ")), right_col[2]);
            },
            2 => { 
                 let body_txt = if app.request_body.is_empty() { "No Body. Press 'b' to open editor." } else { &app.request_body };
                 f.render_widget(Paragraph::new(body_txt).block(config_block.title(" Body Preview ")).wrap(Wrap{trim:true}), right_col[2]);
            },
            3 => { 
                 let title = if app.input_mode == InputMode::EditingAuth { " Bearer Token (Editing) " } else { " Bearer Token (Press 'e' to Edit) " };
                 let style = if app.input_mode == InputMode::EditingAuth { Style::default().fg(Color::Yellow) } else { Style::default() };
                 let auth_txt = if app.auth_token.is_empty() { "No token set" } else { &app.auth_token };
                 f.render_widget(Paragraph::new(auth_txt).block(config_block.title(title).border_style(style)).wrap(Wrap{trim:true}), right_col[2]);
            },
            _ => {}
         };
         
         
         render_response_area(f, app, right_col[3]);
    }

    
    }

    if let Some(msg) = &app.popup_message {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area); 
        let block = Block::default().title(" Notification ").borders(Borders::ALL).style(Style::default().bg(Color::Blue).fg(Color::White));
        let para = Paragraph::new(msg.as_str()).block(block).wrap(Wrap { trim: true }).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, area);
    }

fn render_response_area(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let status_bar_text = if app.is_loading { 
        " Fetching... ".to_string() 
    } else {
        match (app.status_code, app.latency) {
            (Some(code), Some(ms)) => format!(" Status: {} | Time: {}ms ", code, ms),
            (Some(code), None) => format!(" Status: {} ", code),
            _ => " Response ".to_string(),
        }
    };
    
    let status_style = if let Some(code) = app.status_code {
         if code >= 200 && code < 300 { Style::default().fg(Color::Green) } 
         else if code >= 400 { Style::default().fg(Color::Red) } 
         else { Style::default().fg(Color::Yellow) }
    } else { Style::default().fg(Color::Blue) };

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
            .block(Block::default().title(block_title).borders(Borders::ALL).border_style(status_style))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, area, &mut app.json_list_state);
    } else {
         let content = app.response.as_deref().unwrap_or("No data yet. Press Enter to send request.");
         let para = Paragraph::new(content)
            .block(Block::default().title(block_title).borders(Borders::ALL).border_style(status_style))
            .wrap(Wrap{trim:true});
         f.render_widget(para, area);
    }
}
    
    if app.show_help {
        let area = centered_rect(60, 60, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .title(" Help (Press '?' to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
            
        let help_text = vec![
            "General:",
            "  q          Quit",
            "  ?          Toggle Help",
            "  Ctrl+h     Focus Sidebar / Main",
            "  Ctrl+e     Switch Environment",
            "",
            "Navigation:",
            "  j / k      Move Up / Down",
            "  h / l      Collapse / Expand JSON",
            "  Tab        Cycle Tabs (Params, Headers, Body, Auth)",
            "",
            "Request:",
            "  e          Edit URL (Tab to Cycle Method)",
            "  m          Cycle Method (GET, POST, ...)",
            "  b          Edit Body (External Editor)",
            "  H          Edit Headers (External Editor)",
            "  f          Toggle Fullscreen Response",
            "  s          Save Request (to saved.hcl)",
            "  Enter      Send Request",
            "",
            "Tools:",
            "  /          Search / Filter JSON Response",
            "  c          Copy as Curl",
        ].join("\n");

        let para = Paragraph::new(help_text)
             .block(block)
             .style(Style::default().fg(Color::White));
        f.render_widget(para, area);
    }
}



fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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
