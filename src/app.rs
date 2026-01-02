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
    pub json_list_state: ListState,
    pub popup_message: Option<String>,
    
    // Collections
    pub collections: Vec<Collection>,
    pub collection_state: ListState,
    pub active_sidebar: bool, // true if focusing on sidebar
}

use ratatui::widgets::ListState;
use arboard::Clipboard;
use crate::collection::Collection;

impl App {
    pub fn new() -> App {
        let collections = Collection::load_from_dir("collections").unwrap_or_default();
        
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
            
            collections,
            collection_state: ListState::default(),
            active_sidebar: false,
        }
    }

    // Collection Navigation helpers
    pub fn next_collection_item(&mut self) {
        if self.collections.is_empty() { return; }
        
        // Flatten the list for navigation: Collection Header -> Requests -> Next Header
        // For simplicity v1: Just list all requests in a single flat list "Collection::Request"
        
        // Actually, let's just count total requests across all collections for the index
        let total_items = self.flattened_request_count();
        if total_items == 0 { return; }

        let i = match self.collection_state.selected() {
            Some(i) => {
                if i >= total_items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.collection_state.select(Some(i));
    }

    pub fn previous_collection_item(&mut self) {
        let total_items = self.flattened_request_count();
        if total_items == 0 { return; }

        let i = match self.collection_state.selected() {
            Some(i) => {
                if i == 0 {
                    total_items - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.collection_state.select(Some(i));
    }

    pub fn load_selected_request(&mut self) {
        if let Some(idx) = self.collection_state.selected() {
            let req_data = if let Some((_, request)) = self.get_request_at_index(idx) {
                Some((request.url.clone(), request.method.clone()))
            } else {
                None
            };

            if let Some((url, method)) = req_data {
                self.url = url;
                self.method = method.clone();
                self.popup_message = Some(format!("Loaded: {} {}", method, self.url));
            }
        }
    }

    fn flattened_request_count(&self) -> usize {
        self.collections.iter().map(|c| c.requests.len()).sum()
    }

    // Helper to map a flat index back to a specific request
    pub fn get_request_at_index(&self, index: usize) -> Option<(&String, &crate::collection::RequestConfig)> {
        let mut current_idx = 0;
        for col in &self.collections {
            // Sort requests by key to have stable order
            let mut keys: Vec<&String> = col.requests.keys().collect();
            keys.sort();
            
            for key in keys {
                if current_idx == index {
                    return col.requests.get(key).map(|r| (key, r));
                }
                current_idx += 1;
            }
        }
        None
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
