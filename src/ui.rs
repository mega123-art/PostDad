use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap, ListItem, List},

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

fn flatten_tree(entries: &[JsonEntry], list_items: &mut Vec<ListItem<'static>>) {
    for entry in entries {
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
        // Truncate if too long (optional polish)
        
        let display_text = format!("{}{} {}: {}", indent, icon, entry.key, val_str);
        
        let item = ListItem::new(display_text).style(get_style_for_value(&entry.value));
        list_items.push(item);

        if entry.is_expanded {
            flatten_tree(&entry.children, list_items);
        }
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
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

    // --- SIDEBAR ---
    let sidebar_block = Block::default()
        .title(" Collections (Ctrl+h to focus) ")
        .borders(Borders::ALL)
        .border_style(if app.active_sidebar { 
            Style::default().fg(Color::Yellow) 
        } else { 
            Style::default().fg(Color::Cyan) 
        });

    let mut collection_items = Vec::new();
    for col in &app.collections {
        // Sort requests for consistent display
        let mut keys: Vec<&String> = col.requests.keys().collect();
        keys.sort();

        for key in keys {
            let req = &col.requests[key];
            let item = ListItem::new(format!("{} [{}] {}", col.name, req.method, key));
            collection_items.push(item);
        }
    }

    let collection_list = List::new(collection_items)
        .block(sidebar_block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(collection_list, chunks[0], &mut app.collection_state);

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

    // --- REQUEST CONFIG (Tabs) ---
    let titles = vec!["[1] Params", "[2] Headers", "[3] Body", "[4] Auth"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Request "))
        .select(app.selected_tab)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, main_chunks[1]);

    // --- RESPONSE AREA ---
    let response_title = if app.is_loading { " Fetching... " } else { " Response " };
    
    if let Some(tree) = &app.response_json {
        let mut items = Vec::new();
        flatten_tree(tree, &mut items);
        let list = List::new(items)
            .block(Block::default()
                .title(format!("{} (JSON Mode)", response_title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, main_chunks[2], &mut app.json_list_state);
    } else {
        let response_text = app.response.as_deref().unwrap_or("No data yet. Press Enter to send request.");
        
        let response_area = Paragraph::new(response_text)
            .block(Block::default()
                .title(response_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)))
            .wrap(Wrap { trim: true });
        f.render_widget(response_area, main_chunks[2]);
    }

    // Popup Rendering
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
