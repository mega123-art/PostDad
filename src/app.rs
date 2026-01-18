use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    EditingAuth,
    EditingBasicAuthUser,
    EditingBasicAuthPass,
    EditingOAuthUrl,
    EditingOAuthTokenUrl,
    EditingOAuthClientId,
    EditingParamKey,
    EditingParamValue,
    EditingChainKey,
    EditingChainPath,
    EditingFormKey,
    EditingFormValue,
    Search,
    EditingWsUrl,
    EditingWsMessage,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppMode {
    Http,
    WebSocket,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BodyType {
    Raw,
    FormData,
    GraphQL,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthType {
    Bearer,
    Basic,
    OAuth2,
    None,
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

use ratatui::style::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub border: Color,
    pub border_focus: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub highlight: Color,
    pub success: Color,
    pub error: Color,
    pub accent: Color,
}

impl Theme {
    pub fn default_theme() -> Self {
        Theme {
            name: "Default".to_string(),
            background: Color::Reset,
            border: Color::DarkGray,
            border_focus: Color::Cyan,
            text_primary: Color::White,
            text_secondary: Color::Gray,
            highlight: Color::Yellow,
            success: Color::Green,
            error: Color::Red,
            accent: Color::Cyan,
        }
    }

    pub fn matrix() -> Self {
        Theme {
            name: "Matrix".to_string(),
            background: Color::Black,
            border: Color::DarkGray,
            border_focus: Color::Green,
            text_primary: Color::Green,
            text_secondary: Color::DarkGray,
            highlight: Color::LightGreen,
            success: Color::Green,
            error: Color::Red,
            accent: Color::Green,
        }
    }

    pub fn cyberpunk() -> Self {
        Theme {
            name: "Cyberpunk".to_string(),
            background: Color::Black,
            border: Color::Magenta,
            border_focus: Color::Yellow,
            text_primary: Color::Cyan,
            text_secondary: Color::Magenta,
            highlight: Color::Yellow,
            success: Color::LightGreen,
            error: Color::Red,
            accent: Color::LightCyan,
        }
    }

    pub fn dracula() -> Self {
        Theme {
            name: "Dracula".to_string(),
            background: Color::Rgb(40, 42, 54),
            border: Color::Rgb(98, 114, 164),
            border_focus: Color::Rgb(189, 147, 249),
            text_primary: Color::Rgb(248, 248, 242),
            text_secondary: Color::Rgb(98, 114, 164),
            highlight: Color::Rgb(255, 121, 198),
            success: Color::Rgb(80, 250, 123),
            error: Color::Rgb(255, 85, 85),
            accent: Color::Rgb(139, 233, 253),
        }
    }
}

#[derive(PartialEq)]
pub enum EditorMode {
    None,
    Body,
    Headers,
    GraphQLQuery,
    GraphQLVariables,
    PreRequestScript,
}

#[derive(Clone, Debug)]
pub struct RequestLog {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub latency: u128,
    pub body: Option<String>,
}

pub struct App {
    pub url: String,
    pub method: String,
    pub response: Option<String>,
    pub response_json: Option<Vec<JsonEntry>>,
    pub input_mode: InputMode,
    pub selected_tab: usize,
    pub is_loading: bool,
    pub spinner_state: usize,
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

    pub request_history: Vec<RequestLog>,

    pub request_body: String,
    pub body_type: BodyType,
    pub form_data: Vec<(String, String, bool)>,
    pub form_list_state: ListState,

    pub graphql_query: String,
    pub graphql_variables: String,

    pub notification_time: Option<std::time::Instant>,

    pub editor_mode: EditorMode,
    pub request_headers: std::collections::HashMap<String, String>,

    pub auth_token: String,
    pub auth_type: AuthType,
    pub basic_auth_user: String,
    pub basic_auth_pass: String,
    pub oauth_auth_url: String,
    pub oauth_token_url: String,
    pub oauth_client_id: String,

    pub latency_history: Vec<u64>,
    pub zen_mode: bool,

    pub show_help: bool,
    pub fullscreen_response: bool,

    pub params: Vec<(String, String)>,
    pub params_list_state: ListState,

    pub extract_rules: Vec<(String, String)>,
    pub extract_list_state: ListState,

    pub trigger_oauth_flow: bool,
    pub response_scroll: (u16, u16),
    pub cookie_jar: std::collections::HashMap<String, Vec<String>>,

    // WebSocket state
    pub app_mode: AppMode,
    pub ws_url: String,
    pub ws_message_input: String,
    pub ws_messages: Vec<crate::websocket::WsMessage>,
    pub ws_connected: bool,
    pub ws_scroll: usize,

    // Pre-request script
    pub pre_request_script: String,
    pub script_output: Vec<String>,

    // Collection Runner
    pub runner_mode: bool,
    pub runner_result: Option<crate::runner::CollectionRunResult>,
    pub runner_scroll: usize,

    // Splash screen
    pub show_splash: bool,

    // Theme
    pub theme: Theme,
    pub theme_index: usize,
}

use crate::collection::Collection;
use crate::environment::Environment;
use arboard::Clipboard;
use ratatui::widgets::ListState;

impl App {
    pub fn new() -> App {
        let (cols, col_state) = match Collection::load_from_dir("collections") {
            Ok(c) => (c, ListState::default()),
            Err(_) => (Vec::new(), ListState::default()),
        };

        let (envs, env_idx) = match Environment::load_from_file("environments.hcl") {
            Ok(e) => (e, 0),
            Err(_) => (Vec::new(), 0),
        };

        App {
            url: String::from("https://api.github.com/zen"),
            method: String::from("GET"),
            input_mode: InputMode::Normal,
            response: None,
            response_json: None,
            selected_tab: 0,
            is_loading: false,
            spinner_state: 0,
            latency: None,
            status_code: None,
            search_query: String::new(),
            json_list_state: ListState::default(),
            popup_message: None,
            collections: cols,
            collection_state: col_state,
            active_sidebar: false,
            environments: envs,
            selected_env_index: env_idx,
            request_history: Vec::new(),
            request_body: String::new(),
            body_type: BodyType::Raw,
            form_data: Vec::new(),
            form_list_state: ListState::default(),
            graphql_query: String::new(),
            graphql_variables: String::new(),
            notification_time: None,
            editor_mode: EditorMode::None,
            request_headers: std::collections::HashMap::new(),
            auth_token: String::new(),
            auth_type: AuthType::None,
            basic_auth_user: String::new(),
            basic_auth_pass: String::new(),
            oauth_auth_url: String::from("https://github.com/login/oauth/authorize"),
            oauth_token_url: String::from("https://github.com/login/oauth/access_token"),
            oauth_client_id: String::new(),
            latency_history: Vec::new(),
            zen_mode: false,
            show_help: false,
            fullscreen_response: false,
            params: Vec::new(),
            params_list_state: ListState::default(),
            extract_rules: Vec::new(),
            extract_list_state: ListState::default(),
            trigger_oauth_flow: false,
            response_scroll: (0, 0),
            cookie_jar: std::collections::HashMap::new(),

            // WebSocket state
            app_mode: AppMode::Http,
            ws_url: String::from("wss://echo.websocket.org"),
            ws_message_input: String::new(),
            ws_messages: Vec::new(),
            ws_connected: false,
            ws_scroll: 0,

            // Pre-request script
            pre_request_script: String::new(),
            script_output: Vec::new(),

            // Collection Runner
            runner_mode: false,
            runner_result: None,
            runner_scroll: 0,

            // Splash screen
            show_splash: true,

            // Theme
            theme: Theme::default_theme(),
            theme_index: 0,
        }
    }



    pub fn next_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % 4;
        self.theme = match self.theme_index {
            0 => Theme::default_theme(),
            1 => Theme::matrix(),
            2 => Theme::cyberpunk(),
            3 => Theme::dracula(),
            _ => Theme::default_theme(),
        };
    }

    pub fn add_cookies(&mut self, url: &str, new_cookies: Vec<String>) {
        if new_cookies.is_empty() {
            return;
        }

        if let Ok(parsed) = reqwest::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                let entry = self
                    .cookie_jar
                    .entry(host.to_string())
                    .or_insert_with(Vec::new);
                for raw_cookie in new_cookies {
                    let name_val = raw_cookie
                        .split(';')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if name_val.is_empty() {
                        continue;
                    }

                    let name = name_val.split('=').next().unwrap_or("").trim();
                    entry.retain(|c| !c.starts_with(&format!("{}=", name)));

                    entry.push(name_val);
                }
            }
        }
    }

    pub fn get_cookie_header(&self, url: &str) -> Option<String> {
        if let Ok(parsed) = reqwest::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                if let Some(cookies) = self.cookie_jar.get(host) {
                    if !cookies.is_empty() {
                        return Some(cookies.join("; "));
                    }
                }
            }
        }
        None
    }

    pub fn should_open_editor(&self) -> bool {
        self.editor_mode != EditorMode::None
    }

    pub fn get_active_env(&self) -> &Environment {
        &self.environments[self.selected_env_index]
    }

    pub fn next_env(&mut self) {
        if self.environments.is_empty() {
            return;
        }
        self.selected_env_index = (self.selected_env_index + 1) % self.environments.len();
    }

    pub fn process_url(&self) -> String {
        let mut final_url = self.url.clone();
        let env = self.get_active_env();

        for (key, val) in &env.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            final_url = final_url.replace(&placeholder, val);
        }
        final_url
    }

    pub fn sync_url_to_params(&mut self) {
        if let Ok(u) = reqwest::Url::parse(&self.url) {
            self.params = u.query_pairs().into_owned().collect();
        } else {
            self.params.clear();
        }
    }

    pub fn sync_params_to_url(&mut self) {
        if let Ok(mut u) = reqwest::Url::parse(&self.url) {
            u.query_pairs_mut().clear().extend_pairs(&self.params);
            self.url = u.to_string();
        }
    }

    pub fn add_history(
        &mut self,
        method: String,
        url: String,
        duration: u128,
        status: u16,
        body: Option<String>,
    ) {
        let log = RequestLog {
            method,
            url,
            status,
            latency: duration,
            body,
        };
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

    pub fn show_notification(&mut self, msg: String) {
        self.popup_message = Some(msg);
        self.notification_time = Some(std::time::Instant::now());
    }

    pub fn save_current_request(&mut self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let name = format!("Saved Request {}", timestamp);

        let body_type_str = match self.body_type {
            BodyType::Raw => "Raw",
            BodyType::FormData => "FormData",
            BodyType::GraphQL => "GraphQL",
        };

        if let Err(e) = Collection::save_to_file(
            &name,
            &self.method,
            &self.url,
            &self.request_body,
            &self.request_headers,
            &self.extract_rules,
            &self.form_data,
            body_type_str,
            &self.graphql_query,
            &self.graphql_variables,
            &self.pre_request_script,
        ) {
            self.show_notification(format!("Save Failed: {}", e));
        } else {
            self.show_notification("Saved to collections/saved.hcl (Restart to view)".to_string());
        }
    }

    pub fn next_collection_item(&mut self) {
        let total_items = self.flattened_count();
        if total_items == 0 {
            return;
        }

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
        if total_items == 0 {
            return;
        }

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
            let collection_count = self.flattened_collection_only_count();

            if idx > 0 && idx <= collection_count {
                let req_config = if let Some((_, request)) = self.get_request_at_visual_index(idx) {
                    Some(request.clone())
                } else {
                    None
                };

                if let Some(config) = req_config {
                    self.url = config.url;
                    self.method = config.method;
                    self.request_body = config.body.unwrap_or_default();
                    self.request_headers = config.headers.unwrap_or_default();

                    self.extract_rules = config
                        .extract
                        .map(|m| m.into_iter().collect())
                        .unwrap_or_default();
                    self.form_data = config.form_data.unwrap_or_default();
                    self.graphql_query = config.graphql_query.unwrap_or_default();
                    self.graphql_variables = config.graphql_variables.unwrap_or_default();
                    self.pre_request_script = config.pre_request_script.unwrap_or_default();

                    self.body_type = match config.body_type.as_deref() {
                        Some("FormData") => BodyType::FormData,
                        Some("GraphQL") => BodyType::GraphQL,
                        _ => BodyType::Raw,
                    };

                    self.sync_url_to_params();

                    self.show_notification(format!("Loaded: {} {}", self.method, self.url));
                }
            } else if idx > collection_count + 2 {
                let history_idx = idx - (collection_count + 3);
                if history_idx < self.request_history.len() {
                    if let Some(log) = self.request_history.get(history_idx) {
                        self.method = log.method.clone();
                        self.url = log.url.clone();
                        self.status_code = Some(log.status);
                        self.latency = Some(log.latency);
                        self.response = log.body.clone();

                        if let Some(body_text) = &log.body {
                            if let Ok(val) = serde_json::from_str::<Value>(body_text) {
                                let root =
                                    crate::app::JsonEntry::from_value("root".to_string(), &val, 0);
                                self.response_json = Some(vec![root]);
                            } else {
                                self.response_json = None;
                            }
                        } else {
                            self.response_json = None;
                        }

                        self.popup_message = Some("Restored from history".to_string());
                    }
                }
            }
        }
    }

    fn flattened_count(&self) -> usize {
        let cols = self.flattened_collection_only_count();
        let hist = if self.request_history.is_empty() {
            0
        } else {
            self.request_history.len() + 2
        };
        cols + 1 + hist
    }

    fn flattened_collection_only_count(&self) -> usize {
        self.collections.iter().map(|c| c.requests.len()).sum()
    }

    pub fn get_request_at_visual_index(
        &self,
        visual_index: usize,
    ) -> Option<(&String, &crate::collection::RequestConfig)> {
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
        let mut cmd = format!("curl -X {} \"{}\"", self.method, self.process_url());

        match &self.auth_type {
            AuthType::Bearer => {
                if !self.auth_token.is_empty() {
                    cmd.push_str(&format!(
                        " -H \"Authorization: Bearer {}\"",
                        self.auth_token
                    ));
                }
            }
            AuthType::Basic => {
                let creds = format!("{}:{}", self.basic_auth_user, self.basic_auth_pass);
                let _encoded = arboard::Clipboard::new()
                    .map(|_| "ENCODING_SKIPPED".to_string())
                    .unwrap_or("".to_string());
                cmd.push_str(&format!(" --user \"{}\"", creds));
            }
            AuthType::OAuth2 => {
                if !self.auth_token.is_empty() {
                    cmd.push_str(&format!(
                        " -H \"Authorization: Bearer {}\"",
                        self.auth_token
                    ));
                }
            }
            _ => {}
        }

        for (k, v) in &self.request_headers {
            cmd.push_str(&format!(" -H \"{}: {}\"", k, v));
        }

        match self.body_type {
            BodyType::Raw => {
                if !self.request_body.is_empty() {
                    let escaped = self.request_body.replace("'", "'\\''");
                    cmd.push_str(&format!(" -d '{}'", escaped));
                }
            }
            BodyType::FormData => {
                for (k, v, is_file) in &self.form_data {
                    if *is_file {
                        cmd.push_str(&format!(" -F \"{} = @{}\"", k, v));
                    } else {
                        cmd.push_str(&format!(" -F \"{} = {}\"", k, v));
                    }
                }
            }
            BodyType::GraphQL => {
                let vars = if self.graphql_variables.trim().is_empty() {
                    "{}"
                } else {
                    &self.graphql_variables
                };
                let payload = format!(
                    "{{\"query\": \"{}\", \"variables\": {}}}",
                    self.graphql_query
                        .replace("\"", "\\\"")
                        .replace("\n", "\\n"),
                    vars
                );
                let escaped = payload.replace("'", "'\\''");
                cmd.push_str(&format!(" -d '{}'", escaped));
            }
        }

        cmd
    }

    pub fn generate_python_code(&self) -> String {
        let mut code = String::from("import requests\n\n");
        code.push_str(&format!("url = \"{}\"\n", self.process_url()));

        code.push_str("headers = {\n");
        for (k, v) in &self.request_headers {
            code.push_str(&format!("    \"{}\": \"{}\",\n", k, v));
        }
        if self.auth_type == AuthType::Bearer || self.auth_type == AuthType::OAuth2 {
            if !self.auth_token.is_empty() {
                code.push_str(&format!(
                    "    \"Authorization\": \"Bearer {}\",\n",
                    self.auth_token
                ));
            }
        }
        code.push_str("}\n\n");

        match self.body_type {
            BodyType::Raw => {
                if !self.request_body.is_empty() {
                    code.push_str(&format!("payload = '''{}'''\n\n", self.request_body));
                    code.push_str(&format!(
                        "response = requests.request(\"{}\", url, headers=headers, data=payload)",
                        self.method
                    ));
                } else {
                    code.push_str(&format!(
                        "response = requests.request(\"{}\", url, headers=headers)",
                        self.method
                    ));
                }
            }
            BodyType::FormData => {
                code.push_str("files = [\n");
                for (k, v, is_file) in &self.form_data {
                    if *is_file {
                        code.push_str(&format!("    ('{}', open('{}', 'rb')),\n", k, v));
                    } else {
                        code.push_str(&format!("    ('{}', (None, '{}')),\n", k, v));
                    }
                }
                code.push_str("]\n\n");
                code.push_str(&format!(
                    "response = requests.request(\"{}\", url, headers=headers, files=files)",
                    self.method
                ));
            }
            _ => {
                code.push_str(&format!(
                    "response = requests.request(\"{}\", url, headers=headers)",
                    self.method
                ));
            }
        }

        code.push_str("\n\nprint(response.text)");
        code
    }

    pub fn generate_javascript_code(&self) -> String {
        let mut code = format!(
            "const url = \"{}\";\nconst options = {{\n  method: '{}',\n  headers: {{\n",
            self.process_url(),
            self.method
        );

        for (k, v) in &self.request_headers {
            code.push_str(&format!("    '{}': '{}',\n", k, v));
        }
        if self.auth_type == AuthType::Bearer || self.auth_type == AuthType::OAuth2 {
            if !self.auth_token.is_empty() {
                code.push_str(&format!(
                    "    'Authorization': 'Bearer {}',\n",
                    self.auth_token
                ));
            }
        }
        code.push_str("  },\n");

        if self.body_type == BodyType::Raw && !self.request_body.is_empty() {
            code.push_str(&format!("  body: JSON.stringify({})\n", self.request_body));
        } else if self.body_type == BodyType::FormData {
            code.push_str("  body: formData\n");
        }

        code.push_str("};\n\n");

        if self.body_type == BodyType::FormData {
            code.push_str("// Note: Construct FormData manually if needed\n\n");
        }

        code.push_str("try {\n  const response = await fetch(url, options);\n  const data = await response.json();\n  console.log(data);\n} catch (error) {\n  console.error(error);\n}");
        code
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

    fn get_mut_node_at_index<'a>(
        entries: &'a mut Vec<JsonEntry>,
        target_index: &mut usize,
    ) -> Option<&'a mut JsonEntry> {
        for entry in entries {
            if *target_index == 0 {
                return Some(entry);
            }
            *target_index -= 1;

            if entry.is_expanded {
                if let Some(child) = Self::get_mut_node_at_index(&mut entry.children, target_index)
                {
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

    pub fn scroll_down(&mut self) {
        self.response_scroll.0 += 1;
    }

    pub fn scroll_up(&mut self) {
        if self.response_scroll.0 > 0 {
            self.response_scroll.0 -= 1;
        }
    }

    pub fn scroll_page_down(&mut self) {
        if let Some(entries) = &self.response_json {
            let count = Self::count_visible(entries);
            if count > 0 {
                let current = self.json_list_state.selected().unwrap_or(0);
                let next = (current + 10).min(count - 1);
                self.json_list_state.select(Some(next));
                return;
            }
        }
        self.response_scroll.0 += 10;
    }

    pub fn scroll_page_up(&mut self) {
        if let Some(entries) = &self.response_json {
            let count = Self::count_visible(entries);
            if count > 0 {
                let current = self.json_list_state.selected().unwrap_or(0);
                let next = if current > 10 { current - 10 } else { 0 };
                self.json_list_state.select(Some(next));
                return;
            }
        }
        if self.response_scroll.0 > 10 {
            self.response_scroll.0 -= 10;
        } else {
            self.response_scroll.0 = 0;
        }
    }

    pub fn next_item(&mut self) {
        let count = self.calculate_visible_item_count();
        if count == 0 {
            self.scroll_down();
            return;
        }

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
        if count == 0 {
            self.scroll_up();
            return;
        }

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
