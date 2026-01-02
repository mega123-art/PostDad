use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Debug)]
pub struct JsonEntry {
    pub key: String,
    pub value: Value,
    pub level: usize,
    pub is_expanded: bool,
    pub children: Vec<JsonEntry>,
}

impl JsonEntry {
    pub fn from_value(key: String, value: &Value, level: usize) -> Self {
        let mut children = Vec::new();
        
        if let Value::Object(map) = value {
            for (k, v) in map {
                children.push(JsonEntry::from_value(k.clone(), v, level + 1));
            }
        } else if let Value::Array(list) = value {
            for (i, v) in list.iter().enumerate() {
                children.push(JsonEntry::from_value(format!("[{}]", i), v, level + 1));
            }
        }

        JsonEntry {
            key,
            value: value.clone(),
            level,
            is_expanded: true, 
            children,
        }
    }
}

pub struct App {
    pub url: String,
    pub method: String,
    pub response: Option<String>,
    pub response_json: Option<Vec<JsonEntry>>, 
    pub input_mode: InputMode,
    pub selected_tab: usize,
    pub is_loading: bool,
    pub json_list_state: ratatui::widgets::ListState,
    pub popup_message: Option<String>,
}

use ratatui::widgets::ListState;
use arboard::Clipboard;

impl App {
    pub fn new() -> App {
        App {
            url: String::from("https://api.github.com/zen"),
            method: String::from("GET"),
            response: None,
            response_json: None,
            input_mode: InputMode::Normal,
            selected_tab: 0,
            is_loading: false,
            json_list_state: ListState::default(),
            popup_message: None,
        }
    }

    pub fn generate_curl_command(&self) -> String {
        format!("curl -X {} \"{}\"", self.method, self.url)
    }

    pub fn copy_to_clipboard(&mut self, text: String) {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(text) {
                    self.popup_message = Some(format!("Clipboard Error: {}", e));
                } else {
                    self.popup_message = Some("Copied to clipboard!".to_string());
                }
            }
            Err(e) => {
                self.popup_message = Some(format!("Clipboard Init Error: {}", e));
            }
        }
    }

    // Helper to traverse the tree and find the node at the visual index
    pub fn toggle_current_selection(&mut self) {
        if let Some(selected_idx) = self.json_list_state.selected() {
            if let Some(entries) = &mut self.response_json {
                let mut current_idx = selected_idx;
                if let Some(node) = Self::get_mut_node_at_index(entries, &mut current_idx) {
                    node.is_expanded = !node.is_expanded;
                }
            }
        }
    }

    // Explicitly expand or collapse (for Left/Right keys)
    pub fn set_expanded_current_selection(&mut self, expanded: bool) {
         if let Some(selected_idx) = self.json_list_state.selected() {
            if let Some(entries) = &mut self.response_json {
                let mut current_idx = selected_idx;
                if let Some(node) = Self::get_mut_node_at_index(entries, &mut current_idx) {
                    node.is_expanded = expanded;
                }
            }
        }
    }

    fn get_mut_node_at_index<'a>(entries: &'a mut Vec<JsonEntry>, target_index: &mut usize) -> Option<&'a mut JsonEntry> {
        for entry in entries {
            if *target_index == 0 {
                return Some(entry);
            }
            *target_index -= 1;

            if entry.is_expanded {
                if let Some(child) = Self::get_mut_node_at_index(&mut entry.children, target_index) {
                    return Some(child);
                }
            }
        }
        None
    }

    // Determine total visible items to clamp selection
    pub fn calculate_visible_item_count(&self) -> usize {
        if let Some(entries) = &self.response_json {
            Self::count_visible(entries)
        } else {
            0
        }
    }

    fn count_visible(entries: &[JsonEntry]) -> usize {
        let mut count = 0;
        for entry in entries {
            count += 1;
            if entry.is_expanded {
                count += Self::count_visible(&entry.children);
            }
        }
        count
    }

    pub fn next_item(&mut self) {
        let count = self.calculate_visible_item_count();
        if count == 0 { return; }
        
        let i = match self.json_list_state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.json_list_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let count = self.calculate_visible_item_count();
        if count == 0 { return; }

        let i = match self.json_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.json_list_state.select(Some(i));
    }
}
