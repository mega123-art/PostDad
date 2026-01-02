use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Search,
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

#[derive(PartialEq)]
pub enum EditorMode {
    None,
    Body,
    Headers,
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
    
    pub collections: Vec<Collection>,
    pub collection_state: ListState,
    pub active_sidebar: bool, 

    pub latency: Option<u128>,
    pub status_code: Option<u16>, 
    pub search_query: String,
    
    pub environments: Vec<Environment>,
    pub selected_env_index: usize,

    pub request_history: Vec<String>,
    
    pub request_body: String,
    pub editor_mode: EditorMode, // Replaces should_open_editor boolean
    pub request_headers: std::collections::HashMap<String, String>,
    
    pub show_help: bool,
}

use ratatui::widgets::ListState;
use arboard::Clipboard;
use crate::collection::Collection;
use crate::environment::Environment;

impl App {
    pub fn new() -> App {
        let collections = Collection::load_from_dir("collections").unwrap_or_default();
        let environments = Environment::load_from_file("environments.hcl").unwrap_or_default();
        
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
            
            latency: None,
            status_code: None,
            search_query: String::new(),
            
            environments,
            selected_env_index: 0,
            
            request_history: Vec::new(),
            
            request_body: String::new(),
            editor_mode: EditorMode::None,
            request_headers: std::collections::HashMap::new(),
            
            show_help: false,
        }
    }
    
    pub fn should_open_editor(&self) -> bool {
        self.editor_mode != EditorMode::None
    }

    pub fn get_active_env(&self) -> &Environment {
        &self.environments[self.selected_env_index]
    }

    pub fn next_env(&mut self) {
        if self.environments.is_empty() { return; }
        self.selected_env_index = (self.selected_env_index + 1) % self.environments.len();
    }

    pub fn process_url(&self) -> String {
        let mut final_url = self.url.clone();
        let env = self.get_active_env();
        
        for (key, val) in &env.variables {
            let placeholder = format!("{{{{{}}}}}", key); // {{key}}
            final_url = final_url.replace(&placeholder, val);
        }
        final_url
    }

    pub fn add_history(&mut self, method: String, url: String, duration: u128) {
        let log = format!("[{}] {} ({}ms)", method, url, duration);
        self.request_history.insert(0, log);
        if self.request_history.len() > 50 {
            self.request_history.pop();
        }
    }

    pub fn cycle_method(&mut self) {
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
        let current_pos = methods.iter().position(|&m| m == self.method).unwrap_or(0);
        let next = (current_pos + 1) % methods.len();
        self.method = methods[next].to_string();
    }
    
    pub fn trigger_editor(&mut self) {
        self.editor_mode = EditorMode::Body;
    }
    
    pub fn trigger_header_editor(&mut self) {
        self.editor_mode = EditorMode::Headers;
    }

    pub fn save_current_request(&mut self) {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let name = format!("Saved Request {}", timestamp);
        
        if let Err(e) = Collection::save_to_file(&name, &self.method, &self.url, &self.request_body, &self.request_headers) {
             self.popup_message = Some(format!("Save Failed: {}", e));
        } else {
             self.popup_message = Some("Saved to collections/saved.hcl (Restart to view)".to_string());
        }
    }


    pub fn next_collection_item(&mut self) {
        let total_items = self.flattened_count();
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
        let total_items = self.flattened_count();
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

    // This handles both requests AND history for selection
    pub fn load_selected_request(&mut self) {
        if let Some(idx) = self.collection_state.selected() {
            let collection_count = self.flattened_collection_only_count();
            
            // Adjust index to skip the "--- Collections ---" header
            if idx > 0 && idx <= collection_count {
                 // It's a collection item
                 let req_data = if let Some((_, request)) = self.get_request_at_visual_index(idx) {
                     Some((request.url.clone(), request.method.clone()))
                 } else {
                     None
                 };

                 if let Some((url, method)) = req_data {
                     self.url = url;
                     self.method = method;
                     self.popup_message = Some(format!("Loaded: {} {}", self.method, self.url));
                 }
            } else if idx > collection_count + 1 {
                 let history_idx = idx - (collection_count + 2); // 2 headers
                 if history_idx < self.request_history.len() {
                      if let Some(log) = self.request_history.get(history_idx) {
                          let parts: Vec<&str> = log.split_whitespace().collect();
                          if parts.len() >= 2 {
                              self.method = parts[0].trim_matches(|c| c == '[' || c == ']').to_string();
                              self.url = parts[1].to_string();
                              self.popup_message = Some("Restored from history".to_string());
                          }
                      }
                 }
            }
        }
    }

    fn flattened_count(&self) -> usize {
        let cols = self.flattened_collection_only_count();
        let hist = if self.request_history.is_empty() { 0 } else { self.request_history.len() + 1 };
        cols + 1 + hist 
    }

    fn flattened_collection_only_count(&self) -> usize {
        self.collections.iter().map(|c| c.requests.len()).sum()
    }

    pub fn get_request_at_visual_index(&self, visual_index: usize) -> Option<(&String, &crate::collection::RequestConfig)> {
        let mut current = 1; 
        for col in &self.collections {
            let mut keys: Vec<&String> = col.requests.keys().collect();
            keys.sort();
            for key in keys {
                if current == visual_index {
                    return col.requests.get(key).map(|r| (key, r));
                }
                current += 1;
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
