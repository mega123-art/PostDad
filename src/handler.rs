use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    // Global Shortcuts
    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            KeyCode::Char('h') => {
                app.active_sidebar = !app.active_sidebar;
                if app.active_sidebar {
                    // Select first item if nothing selected
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
            _ => {}
        }
    }

    if app.active_sidebar {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => app.next_collection_item(),
            KeyCode::Char('k') | KeyCode::Up => app.previous_collection_item(),
            KeyCode::Enter => {
                app.load_selected_request();
                // Optionally close sidebar or keep it open? Let's keep focus for rapid testing
            }
            KeyCode::Esc => app.active_sidebar = false,
            _ => {}
        }
        return;
    }

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
            KeyCode::Char('m') => {
                app.cycle_method();
            }
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                app.next_item();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.previous_item();
            }
            // Tree Expansion

            KeyCode::Char('l') | KeyCode::Right => {
                app.set_expanded_current_selection(true);
            }
            KeyCode::Char(' ') => {
                 // Space also works for editor if on Body tab? No, keep it for tree. 
                 // Let's use 'e' for editor if NOT in search/editing URL?
                 // Wait, 'e' is for editing URL.
                 // Let's use Enter. If selected_tab == 2 (Body), open editor.
                 // But Enter currently sends request.
                 // Conflict! 
                 // Solution: 'e' focuses URL bar. 
                 // Maybe 'b' for Body Editor?
                 // Or stick to Enter on Body Tab, and use Ctrl+Enter to send request?
                 // Or, if tab is Body, and user presses 'e', open editor?
                 // Let's use 'b' for Body Edit shortcut for now, simple.
                 app.toggle_current_selection();
            }
            KeyCode::Char('b') => {
                app.selected_tab = 2; // Jump to body
                app.trigger_editor();
            }
            KeyCode::Char('?') => {
                app.show_help = !app.show_help;
            }
            KeyCode::Char('h') | KeyCode::Left => { // Override 'h' for editor if in headers tab?
                 // Conflict with Left navigation. 
                 // Vim users expect h/j/k/l for nav.
                 // Let's use 'H' (Shift+h) for Headers Editor just to be safe, or check modifiers.
                 // Or stick to Context: if selected_tab == 1 (Headers), then 'e' or 'Enter' opens editor?
                 // Let's make 'h' navigation only.
                 // And add 'H' for headers.
                 app.set_expanded_current_selection(false);
            }
            KeyCode::Char('H') => {
                app.selected_tab = 1; // Jump to headers
                app.trigger_header_editor();
            }
            KeyCode::Char('s') => {
                app.save_current_request();
            }
            KeyCode::Char('c') => {
                let cmd = app.generate_curl_command();
                app.copy_to_clipboard(cmd);
            }
            KeyCode::Esc => {
                app.popup_message = None;
            }
            KeyCode::Char('/') => {
                app.input_mode = InputMode::Search;
                app.search_query.clear();
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
    }
}
