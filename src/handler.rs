use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    
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
            _ => {}
        }
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
                if app.selected_tab == 3 {
                    app.input_mode = InputMode::EditingAuth;
                } else {
                    app.input_mode = InputMode::Editing;
                }
            }
            KeyCode::Char('q') => {
                
            }
            KeyCode::Enter => {
                
            }
            KeyCode::Tab => {
                app.selected_tab = (app.selected_tab + 1) % 4;
            }
            KeyCode::Char('m') => {
                app.cycle_method();
            }
            
            KeyCode::Char('j') | KeyCode::Down => {
                app.next_item();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.previous_item();
            }
            

            KeyCode::Char('l') | KeyCode::Right => {
                app.set_expanded_current_selection(true);
            }
            KeyCode::Char(' ') => {
                 
                 
                 
                 
                 
                 
                 
                 
                 
                 
                 
                 app.toggle_current_selection();
            }
            KeyCode::Char('b') => {
                app.selected_tab = 2; 
                app.trigger_editor();
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
            KeyCode::Char('f') => {
                app.fullscreen_response = !app.fullscreen_response;
            }
            _ => {}
        },
        InputMode::Editing => match key_event.code {
            KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
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
    }
}
