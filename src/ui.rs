use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap, ListItem, List},
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
        // Filter Logic:
        // If filter is empty, show everything (respecting expansion).
        // If filter is NOT empty, show item ONLY if key contains filter.
        // (Advanced: Show if children match? For v1, simple key filtering)
        
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

            // Shorten value for display
            let val_str = match &entry.value {
                 serde_json::Value::String(s) => format!("\"{}\"", s), // naive quoting
                 v => format!("{}", v),
            };
            
            let display_text = format!("{}{} {}: {}", indent, icon, entry.key, val_str);
            
            let item = ListItem::new(display_text).style(get_style_for_value(&entry.value));
            list_items.push(item);
        }

        // Recursion logic with filter
        // If filter is active, we might want to auto-expand or just search deep
        // For now, respect expansion unless filtering, but if filtering, maybe search all?
        // Let's stick to visible structure for now to keep it "tree-like"
        if entry.is_expanded {
            flatten_tree(&entry.children, list_items, filter);
        }
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
    // 1. Define the Main Areas
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Sidebar
            Constraint::Percentage(80), // Main Content
        ])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header/Tabs (Keep structure simple)
            Constraint::Min(10),        // Content
        ])
        .split(chunks[1]);

    // --- SIDEBAR ---
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
    
    // 1. Collections
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
    
    // 2. History
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
    
    f.render_stateful_widget(collection_list, chunks[0], &mut app.collection_state);

    // --- URL BAR ---
    let url_border_color = match app.input_mode {
        InputMode::Editing => Color::Yellow,
        InputMode::Search => Color::Magenta,
        _ => Color::Blue, // Standard Blue
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
    
    // REDO LAYOUT Logic inside render (keeping the fix for visibility, but simplifying styles)
    let right_col = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // URL
            Constraint::Length(3),      // Tabs Header
            Constraint::Length(8),      // Config Content / Body Preview
            Constraint::Min(10),        // Response
        ])
        .split(chunks[1]);

    // 1. URL
    f.render_widget(url_bar, right_col[0]);

    // 2. Tabs
    let titles = vec![
        " [1] Params ", " [2] Headers ", " [3] Body (b) ", " [4] Auth "
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app.selected_tab)
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, right_col[1]);
    
    // 3. Config Content
    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
        
    match app.selected_tab {
        0 => { // Params
             let info = Paragraph::new("Query Parameters currently must be edited directly in the URL bar above.\n\nTip: Press 'e' to edit the URL.")
                .block(config_block.title(" Params "))
                .wrap(Wrap{trim:true});
             f.render_widget(info, right_col[2]);
        },
        1 => { // Headers
            let headers: Vec<ListItem> = app.request_headers.iter()
                .map(|(k,v)| ListItem::new(format!("{}: {}", k, v))).collect();
            let list = List::new(headers).block(config_block.title(" Headers "));
            f.render_widget(list, right_col[2]);
        },
        2 => { // Body
             let body_txt = if app.request_body.is_empty() { "No Body. Press 'b' to open editor." } else { &app.request_body };
             let para = Paragraph::new(body_txt).block(config_block.title(" Body Preview ")).wrap(Wrap{trim:true});
             f.render_widget(para, right_col[2]);
        },
        3 => { // Auth
             let info = Paragraph::new("Authentication helpers are coming soon.\n\nPlease use the [2] Headers tab to manually set 'Authorization'.\n\nShortcut: Press 'H' to edit headers as JSON.")
                .block(config_block.title(" Auth "))
                .wrap(Wrap{trim:true});
             f.render_widget(info, right_col[2]);
        },
        _ => {}
    };

    // 4. Response Area
    let status_bar_text = if app.is_loading { 
        " Fetching... ".to_string() 
    } else {
        match (app.status_code, app.latency) {
            (Some(code), Some(ms)) => format!(" Status: {} | Time: {}ms ", code, ms),
            (Some(code), None) => format!(" Status: {} ", code),
            _ => " Response ".to_string(),
        }
    };
    
    // Status Color Logic
    let status_style = if let Some(code) = app.status_code {
        if code >= 200 && code < 300 {
            Style::default().fg(Color::Green)
        } else if code >= 400 {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default().fg(Color::Blue) // Default border color
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
            .block(Block::default()
                .title(block_title)
                .borders(Borders::ALL)
                .border_style(status_style))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, right_col[3], &mut app.json_list_state);
    } else {
         let content = app.response.as_deref().unwrap_or("No data yet. Press Enter to send request.");
         let para = Paragraph::new(content)
            .block(Block::default()
                .title(block_title)
                .borders(Borders::ALL)
                .border_style(status_style))
            .wrap(Wrap{trim:true});
         f.render_widget(para, right_col[3]);
    }

    // Popup Rendering
    if let Some(msg) = &app.popup_message {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area); 
        let block = Block::default().title(" Notification ").borders(Borders::ALL).style(Style::default().bg(Color::Blue).fg(Color::White));
        let para = Paragraph::new(msg.as_str()).block(block).wrap(Wrap { trim: true }).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, area);
    }

    // Help Popup (Clean style)
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
            "  e          Edit URL",
            "  m          Cycle Method (GET, POST, ...)",
            "  b          Edit Body (External Editor)",
            "  H          Edit Headers (External Editor)",
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
