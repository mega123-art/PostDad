use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

pub fn render_sentinel_mode(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Percentage(50), // Graphs
            Constraint::Percentage(20), // Stats
            Constraint::Min(0), // Log
        ])
        .split(f.area());

    // Header
    let running_status = if let Some(state) = &app.sentinel_state {
        if state.is_running { "RUNNING" } else { "STOPPED" }
    } else {
        "STOPPED"
    };

    let color = if running_status == "RUNNING" { Color::Green } else { Color::Red };
    
    let title = Paragraph::new(format!(" SENTINEL MODE - {} ", running_status))
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if let Some(state) = &app.sentinel_state {
        // Graphs Area
        let graph_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Latency Sparkline
        let latency_data: Vec<u64> = state.latency_history.iter().cloned().collect();
        let latency_sparkline = Sparkline::default()
            .block(Block::default().title(" Latency History (ms) ").borders(Borders::ALL))
            .data(&latency_data)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(latency_sparkline, graph_chunks[0]);

        // Status Code Distribution (Simplified as gauge or text)
        // Let's use a list of recent updates
        let status_text = state.status_history.iter()
            .rev()
            .take(20)
            .map(|s| {
                let style = if *s >= 200 && *s < 300 {
                    Style::default().fg(Color::Green)
                } else if *s >= 400 {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Yellow)
                };
                Line::from(Span::styled(format!("HTTP {}", s), style))
            })
            .collect::<Vec<_>>();
        
        let status_list = Paragraph::new(status_text)
            .block(Block::default().title(" Recent Status ").borders(Borders::ALL));
        f.render_widget(status_list, graph_chunks[1]);

        // Stats Area
        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)])
            .split(chunks[2]);

        let total_checks = Paragraph::new(format!("{}", state.total_checks))
            .block(Block::default().title(" Total Checks ").borders(Borders::ALL))
            .alignment(Alignment::Center);

        // Alert Logic: If failed checks > 0, make it red. If last status was bad, maybe flash?
        // Using red fg for failure count is good.
        let fail_style = if state.failed_checks > 0 { 
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::Green) 
        };
        
        // Check last status for immediate alert
        let last_failed = if let Some(last) = state.last_status {
            last >= 400
        } else {
            false
        };
        
        let fail_block_style = if last_failed {
            Style::default().fg(Color::Red) // Red border if last failed
        } else {
             Style::default()
        };

        let failed_checks = Paragraph::new(format!("{}", state.failed_checks))
            .block(Block::default().title(" Failed Checks ").borders(Borders::ALL).border_style(fail_block_style))
            .style(fail_style)
            .alignment(Alignment::Center);

        let last_latency = Paragraph::new(format!("{} ms", state.last_latency))
            .block(Block::default().title(" Last Latency ").borders(Borders::ALL))
            .alignment(Alignment::Center);

        // Interval Setting
        let interval_style = if app.active_tab().input_mode == crate::app::InputMode::EditingSentinelInterval {
             Style::default().fg(Color::Yellow)
        } else {
             Style::default()
        };
        
        let interval_widget = Paragraph::new(format!("{} s", app.sentinel_interval_input))
             .block(Block::default().title(" Interval (Press 'i') ").borders(Borders::ALL).border_style(interval_style))
             .alignment(Alignment::Center);

        f.render_widget(total_checks, stats_chunks[0]);
        f.render_widget(failed_checks, stats_chunks[1]);
        f.render_widget(last_latency, stats_chunks[2]);
        f.render_widget(interval_widget, stats_chunks[3]);

        // Footer Log/Help
        let footer_text = if app.active_tab().input_mode == crate::app::InputMode::EditingSentinelInterval {
             "Editing Interval... Press Enter to Apply, Esc to Cancel"
        } else {
             "Press 'S' to Stop/Start | 'i' Set Interval | 'L' Save Log | 'Esc' Exit"
        };
        
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::TOP))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[3]);
        
        // Render alert flash?
        if last_failed {
             // We can render a discreet "ALERT" badge in the corner
             let alert = Paragraph::new(" ⚠️ ALERT ")
                 .style(Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD));
             let area = f.area();
             let alert_area = ratatui::layout::Rect { x: area.width.saturating_sub(10), y: 0, width: 10, height: 1 };
             f.render_widget(alert, alert_area);
        }

    } else {
        // Not initialized state (shouldn't happen if logic is correct)
        let msg = Paragraph::new("Sentinel Not Initialized. Press 'S' to Start.")
            .alignment(Alignment::Center);
        f.render_widget(msg, chunks[1]);
    }
}
