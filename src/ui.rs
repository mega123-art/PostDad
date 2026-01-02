use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap, ListItem, List},
    text::Span,
    Frame,
};
use crate::app::{App, InputMode, JsonEntry};

fn get_style_for_value(value: &serde_json::Value) -> Style {
    match value {
        serde_json::Value::String(_) => Style::default().fg(Color::Green),
        serde_json::Value::Number(_) => Style::default().fg(Color::Yellow),
        serde_json::Value::Bool(_) => Style::default().fg(Color::Magenta), // Purpleish
        serde_json::Value::Null => Style::default().fg(Color::DarkGray),
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
    // ... (Layout splitting logic same as before, see lines 52-68 of original) ...
    // 1. Define the Main Areas (Sidebar vs Content)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Sidebar
            Constraint::Percentage(80), // Main Content
        ])
        .split(f.area());

    // 2. Split the Main Content (Top: Request, Bottom: Response)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // URL Bar
            Constraint::Percentage(40), // Request Config (Headers/Body)
            Constraint::Min(10),        // Response Area
        ])
        .split(chunks[1]);

    // ... (Sidebar Rendering lines 70-98, unmodified) ...
    // --- SIDEBAR ---
    let sidebar_title = format!(" {} (Ctrl+e to switch Env) ", app.get_active_env().name);
    let sidebar_block = Block::default()
        .title(sidebar_title)
        .borders(Borders::ALL)
        .border_style(if app.active_sidebar { 
            Style::default().fg(Color::Yellow) 
        } else { 
            Style::default().fg(Color::Cyan) 
        });

    let mut collection_items = Vec::new();
    
    // For now, always show collections on top, history on bottom? Or just mix them? 
    // Let's stick to Collections for now, but if active_env is not None, maybe show variables?
    // Actually, Prompt asked for "Request History". 
    // Let's create a dedicated simplified view for now: "Collections" on top half, "History" on bottom half?
    // Or just append History to the list?
    // Let's append History as a separate section.
    
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

    // ... (URL Bar Rendering lines 101-115, unmodified) ...
    // --- URL BAR (with dynamic styling) ---
    let url_style = match app.input_mode {
        InputMode::Editing => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };
    
    let url_bar = Paragraph::new(app.url.as_str())
        .block(Block::default()
            .title(" URL (Press 'e' to edit, 'Enter' to fetch) ")
            .borders(Borders::ALL)
            .border_style(url_style));
    f.render_widget(url_bar, main_chunks[0]);

    // ... (Tabs Rendering lines 118-124, unmodified) ...
    // --- REQUEST CONFIG (Tabs) ---
    let titles = vec!["[1] Params", "[2] Headers", "[3] Body", "[4] Auth"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Request "))
        .select(app.selected_tab)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, main_chunks[1]);

    // --- RESPONSE AREA ---
    let status_title = if app.is_loading { 
        " Fetching... ".to_string() 
    } else {
        match app.latency {
            Some(ms) => format!(" Response ({}ms) ", ms),
            None => " Response ".to_string(),
        }
    };
    
    // Search Bar Overlay (bottom of response or subtitle)
    let block_title = if app.input_mode == InputMode::Search {
        format!("{} [Search: {}] ", status_title, app.search_query)
    } else if !app.search_query.is_empty() {
        format!("{} [Filter: {}] ", status_title, app.search_query)
    } else {
        status_title
    };

    if let Some(tree) = &app.response_json {
        let mut items = Vec::new();
        flatten_tree(tree, &mut items, &app.search_query);
        let list = List::new(items)
            .block(Block::default()
                .title(block_title)
                .borders(Borders::ALL)
                .border_style(if app.input_mode == InputMode::Search {
                     Style::default().fg(Color::Magenta) 
                } else {
                     Style::default().fg(Color::Green)
                }))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, main_chunks[2], &mut app.json_list_state);
    } else {
        let response_text = app.response.as_deref().unwrap_or("No data yet. Press Enter to send request.");
        
        let response_area = Paragraph::new(response_text)
            .block(Block::default()
                .title(block_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)))
            .wrap(Wrap { trim: true });
        f.render_widget(response_area, main_chunks[2]);
    }
    
    // ... (Popup Rendering - Keep existing) ...
    if let Some(msg) = &app.popup_message {
        let block = Block::default().title(" Notification ").borders(Borders::ALL);
        let area = centered_rect(60, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area); // Clear background
        
        let para = Paragraph::new(msg.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White).bg(Color::Blue));
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
