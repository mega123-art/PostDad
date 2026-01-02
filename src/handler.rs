use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    match app.input_mode {
        InputMode::Normal => match key_event.code {
            KeyCode::Char('e') => {
                app.input_mode = InputMode::Editing;
            }
            KeyCode::Char('q') => {
                // Main loop handles quit
            }
            KeyCode::Enter => {
                // Main loop handles request trigger
            }
            KeyCode::Tab => {
                app.selected_tab = (app.selected_tab + 1) % 4;
            }
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                app.next_item();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.previous_item();
            }
            // Tree Expansion
            KeyCode::Char('h') | KeyCode::Left => {
                app.set_expanded_current_selection(false);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                app.set_expanded_current_selection(true);
            }
            KeyCode::Char(' ') => {
                app.toggle_current_selection();
            }
            KeyCode::Char('c') => {
                let cmd = app.generate_curl_command();
                app.copy_to_clipboard(cmd);
            }
            KeyCode::Esc => {
                app.popup_message = None;
            }
            _ => {}
        },
        InputMode::Editing => match key_event.code {
            KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
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
    }
}
