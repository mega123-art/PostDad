use serde_json::Value;
use serde::{Deserialize, Serialize};
use ratatui_image::picker::Picker;
use image::DynamicImage;

#[derive(Clone, Copy, Debug, PartialEq)]
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
    EditingGrpcService,
    EditingGrpcProto,
    FilteringSidebar,
    CommandPalette,
    Command,
    EditingStressVUs,
    EditingStressDuration,
    EditingSentinelInterval,
    ImportCurl,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppMode {
    Http,
    WebSocket,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyType {
    Raw,
    FormData,
    GraphQL,
    Grpc,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EditorMode {
    None,
    Body,
    Headers,
    GraphQLQuery,
    GraphQLVariables,
    PreRequestScript,
    PostRequestScript,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestLog {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub latency: u128,
    pub body: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
    #[serde(skip)]
    pub response_bytes: Option<Vec<u8>>,
    pub is_binary: bool,
}

#[derive(Clone, Debug)]
pub struct RequestTab {
    pub name: String,

    // Core Request
    pub url: String,
    pub method: String,
    pub input_mode: InputMode,
    pub request_body: String,
    pub body_type: BodyType,
    pub form_data: Vec<(String, String, bool)>,
    pub form_list_state: ListState,
    pub params: Vec<(String, String)>,
    pub params_list_state: ListState,
    pub request_headers: std::collections::HashMap<String, String>,
    pub extract_rules: Vec<(String, String)>,
    pub extract_list_state: ListState,

    // Auth
    pub auth_type: AuthType,
    pub auth_token: String,
    pub basic_auth_user: String,
    pub basic_auth_pass: String,
    pub oauth_auth_url: String,
    pub oauth_token_url: String,
    pub oauth_client_id: String,
    pub trigger_oauth_flow: bool,

    // GraphQL
    pub graphql_query: String,
    pub graphql_variables: String,
    pub graphql_schema_types: Vec<String>,
    pub show_schema_modal: bool,
    pub should_introspect_schema: bool,
    
    // gRPC
    pub grpc_service: String,
    pub grpc_method: String,
    pub grpc_proto_path: String,
    pub grpc_services: Vec<String>,
    pub grpc_service_description: String,
    pub should_list_grpc_services: bool,
    pub show_grpc_services_modal: bool,
    pub should_describe_grpc_service: bool,
    pub grpc_service_to_describe: String,
    pub show_grpc_description_modal: bool,

    // Scripts
    pub pre_request_script: String,
    pub post_request_script: String,
    pub script_output: Vec<String>,
    pub test_results: Vec<(String, bool)>,

    // Response
    pub response: Option<String>,
    pub response_bytes: Option<Vec<u8>>,
    pub response_is_binary: bool,
    pub response_image: Option<DynamicImage>,
    pub response_json: Option<Vec<JsonEntry>>,
    pub response_headers: std::collections::HashMap<String, String>,
    pub status_code: Option<u16>,
    pub latency: Option<u128>,
    pub latency_history: Vec<u64>,
    pub is_loading: bool,
    pub timeout_ms: u64,

    // UI State
    pub selected_tab: usize,
    pub json_list_state: ListState,
    pub search_query: String,
    pub fullscreen_response: bool,
    pub response_scroll: (u16, u16),

    // WebSocket
    pub app_mode: AppMode,
    pub ws_url: String,
    pub ws_message_input: String,
    pub ws_messages: Vec<crate::websocket::WsMessage>,
    pub ws_connected: bool,
    pub ws_scroll: usize,
}

impl RequestTab {
    pub fn new() -> Self {
        RequestTab {
            name: "New Request".to_string(),
            url: String::from("https://api.github.com/zen"), // Default for TAB 1
            method: String::from("GET"),
            input_mode: InputMode::Normal,

            request_body: String::new(),
            body_type: BodyType::Raw,
            form_data: Vec::new(),
            form_list_state: ListState::default(),
            params: Vec::new(),
            params_list_state: ListState::default(),
            request_headers: std::collections::HashMap::new(),
            extract_rules: Vec::new(),
            extract_list_state: ListState::default(),

            auth_type: AuthType::None,
            auth_token: String::new(),
            basic_auth_user: String::new(),
            basic_auth_pass: String::new(),
            oauth_auth_url: String::from("https://github.com/login/oauth/authorize"),
            oauth_token_url: String::from("https://github.com/login/oauth/access_token"),
            oauth_client_id: String::new(),
            trigger_oauth_flow: false,

            graphql_query: String::new(),
            graphql_variables: String::new(),
            graphql_schema_types: Vec::new(),
            show_schema_modal: false,
            should_introspect_schema: false,

            grpc_service: String::new(),
            grpc_method: String::new(),
            grpc_proto_path: String::new(),
            grpc_services: Vec::new(),
            grpc_service_description: String::new(),
            should_list_grpc_services: false,
            show_grpc_services_modal: false,
            should_describe_grpc_service: false,
            grpc_service_to_describe: String::new(),
            show_grpc_description_modal: false,

            pre_request_script: String::new(),
            post_request_script: String::new(),
            script_output: Vec::new(),
            test_results: Vec::new(),

            response: None,
            response_bytes: None,
            response_is_binary: false,
            response_image: None,
            response_json: None,
            response_headers: std::collections::HashMap::new(),
            status_code: None,
            latency: None,
            latency_history: Vec::new(),
            is_loading: false,
            timeout_ms: 30000, // Default 30 seconds

            selected_tab: 0,
            json_list_state: ListState::default(),
            search_query: String::new(),
            fullscreen_response: false,
            response_scroll: (0, 0),

            app_mode: AppMode::Http,
            ws_url: String::from("wss://echo.websocket.org"),
            ws_message_input: String::new(),
            ws_messages: Vec::new(),
            ws_connected: false,
            ws_scroll: 0,
        }
    }

    pub fn clear_response(&mut self) {
        self.response = None;
        self.response_bytes = None;
        self.response_is_binary = false;
        self.response_image = None;
        self.response_json = None;
        self.response_headers.clear();
        self.status_code = None;
        self.latency = None;
        self.script_output.clear();
        self.test_results.clear();
    }
}

pub struct App {
    // Global State
    pub spinner_state: usize,
    pub popup_message: Option<String>,

    pub collections: Vec<Collection>,
    pub collection_state: ListState,
    pub active_sidebar: bool,
    pub sidebar_filter: String,
    pub show_sidebar_filter: bool,

    pub environments: Vec<Environment>,
    pub selected_env_index: usize,

    pub request_history: Vec<RequestLog>,

    pub notification_time: Option<std::time::Instant>,

    pub editor_mode: EditorMode,

    pub zen_mode: bool,
    pub show_help: bool,
    pub help_scroll: u16,

    pub show_command_palette: bool,
    pub command_query: String,
    pub command_index: usize,
    pub command_input: String,

    
    pub show_cookie_modal: bool,
    pub cookie_list_state: ListState,

    pub cookie_jar: std::collections::HashMap<String, Vec<String>>,

    // Tabs
    pub tabs: Vec<RequestTab>,
    pub active_tab: usize,
    pub next_request_id: usize,

    // Collection Runner (Global)
    pub runner_mode: bool,
    pub runner_result: Option<crate::runner::CollectionRunResult>,
    pub runner_scroll: usize,

    // Splash screen
    pub show_splash: bool,

    // Theme
    pub theme: Theme,
    pub theme_index: usize,

    // Diff
    pub diff_base_index: Option<usize>,
    pub show_diff_view: bool,
    pub diff_target_index: Option<usize>,
    pub diff_list_state: ListState,

    // Mock Server
    pub mock_mode: bool,
    pub mock_server_running: bool,
    pub mock_server_port: u16,
    pub mock_routes: Vec<crate::mock_server::MockRoute>,
    pub mock_list_state: ListState,
    pub mock_server_handle: Option<crate::mock_server::MockServerHandle>,
    pub image_picker: Option<Picker>,
    pub clipboard: Option<Clipboard>,

    // Stress Testing State
    pub show_stress_modal: bool,
    pub stress_vus_input: String,
    pub stress_duration_input: String,
    pub stress_running: bool,
    pub stress_stats: Option<crate::stress::StressStats>,
    pub stress_progress: Option<(u64, u64)>,
    pub should_run_stress_test: bool,

    // SSL Configuration
    pub ssl_verify: bool,                    // Whether to verify SSL certificates
    pub ssl_ca_cert_path: Option<String>,    // Path to custom CA certificate
    pub ssl_client_cert_path: Option<String>, // Path to client certificate (for mTLS)
    pub ssl_client_key_path: Option<String>,  // Path to client key (for mTLS)

    // Proxy Configuration
    pub proxy_url: Option<String>,           // HTTP/HTTPS proxy URL (e.g., http://proxy:8080)
    pub proxy_auth_user: Option<String>,     // Proxy authentication username
    pub proxy_auth_pass: Option<String>,     // Proxy authentication password
    pub no_proxy: Option<String>,            // Comma-separated list of hosts to bypass proxy

    // cURL Import
    pub curl_import_input: String,           // Input field for curl command

    // Sentinel Mode
    pub sentinel_mode: bool,
    pub sentinel_state: Option<crate::sentinel::SentinelState>,
    pub should_start_sentinel: bool,
    pub sentinel_interval_input: String,
}

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    theme_index: usize,
    selected_env_index: usize,
    zen_mode: bool,
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

        let mut app = App {
            spinner_state: 0,
            popup_message: None,
            collections: cols,
            collection_state: col_state,
            active_sidebar: false,
            sidebar_filter: String::new(),
            show_sidebar_filter: false,
            environments: envs,
            selected_env_index: env_idx,
            request_history: App::load_history(),
            notification_time: None,
            editor_mode: EditorMode::None,
            zen_mode: false,
            show_help: false,
            help_scroll: 0,
            show_command_palette: false,
            command_query: String::new(),
            command_index: 0,
            command_input: String::new(),
            cookie_jar: std::collections::HashMap::new(),

            tabs: vec![RequestTab::new()],
            active_tab: 0,
            next_request_id: 1,

            runner_mode: false,
            runner_result: None,
            runner_scroll: 0,
            show_splash: true,
            theme: Theme::default_theme(),
            theme_index: 0,

            diff_base_index: None,
            show_diff_view: false,
            diff_target_index: None,
            diff_list_state: ListState::default(),

            mock_mode: false,
            mock_server_running: false,
            mock_server_port: 3000,
            mock_routes: Vec::new(),
            mock_list_state: ListState::default(),
            mock_server_handle: None,
            image_picker: if std::env::var("TERM_PROGRAM").map(|v| v == "vscode").unwrap_or(false) {
                Some(Picker::halfblocks())
            } else {
                Picker::from_query_stdio().ok().or(Some(Picker::halfblocks()))
            },
            clipboard: Clipboard::new().ok(),
            
            show_stress_modal: false,
            stress_vus_input: "50".to_string(), // Default 50 VUs
            stress_duration_input: "10".to_string(), // Default 10s
            stress_running: false,
            stress_stats: None,
            stress_progress: None,
            should_run_stress_test: false,

            show_cookie_modal: false,
            cookie_list_state: ListState::default(),

            // SSL: Load from environment variables or use defaults
            ssl_verify: std::env::var("POSTDAD_SSL_VERIFY")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            ssl_ca_cert_path: std::env::var("POSTDAD_CA_CERT").ok(),
            ssl_client_cert_path: std::env::var("POSTDAD_CLIENT_CERT").ok(),
            ssl_client_key_path: std::env::var("POSTDAD_CLIENT_KEY").ok(),

            // Proxy: Load from standard environment variables
            proxy_url: std::env::var("HTTPS_PROXY")
                .or_else(|_| std::env::var("https_proxy"))
                .or_else(|_| std::env::var("HTTP_PROXY"))
                .or_else(|_| std::env::var("http_proxy"))
                .ok(),
            proxy_auth_user: std::env::var("POSTDAD_PROXY_USER").ok(),
            proxy_auth_pass: std::env::var("POSTDAD_PROXY_PASS").ok(),
            no_proxy: std::env::var("NO_PROXY")
                .or_else(|_| std::env::var("no_proxy"))
                .ok(),

            curl_import_input: String::new(),
            
            sentinel_mode: false,
            sentinel_state: Some(crate::sentinel::SentinelState::new()),
            should_start_sentinel: false,
            sentinel_interval_input: "2".to_string(),
        };

        // Load persisted config and state
        let config = App::load_config();
        app.theme_index = config.theme_index;
        app.zen_mode = config.zen_mode;
        
        // Bounds check env index
        if config.selected_env_index < app.environments.len() {
            app.selected_env_index = config.selected_env_index;
        } else {
            app.selected_env_index = 0;
        }

        app.cookie_jar = App::load_cookies();
        app.request_history = App::load_history();
        
        // Apply loaded theme
        app.apply_theme();

        app
    }

    pub fn active_tab(&self) -> &RequestTab {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut RequestTab {
        &mut self.tabs[self.active_tab]
    }

    pub fn apply_theme(&mut self) {
        self.theme = match self.theme_index {
            0 => Theme::default_theme(),
            1 => Theme::matrix(),
            2 => Theme::cyberpunk(),
            3 => Theme::dracula(),
            _ => Theme::default_theme(),
        };
    }

    pub fn next_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % 4;
        self.apply_theme();
        self.save_config();
    }

    pub fn add_tab(&mut self) {
        let mut tab = RequestTab::new();
        tab.name = format!("Req {}", self.next_request_id);
        self.next_request_id += 1;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.len() > 1 {
            if self.active_tab == 0 {
                self.active_tab = self.tabs.len() - 1;
            } else {
                self.active_tab -= 1;
            }
        }
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
                self.save_cookies();
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

    pub fn get_flattened_cookies(&self) -> Vec<(String, String)> {
        let mut flattened = Vec::new();
        for (host, cookies) in &self.cookie_jar {
            for cookie in cookies {
                flattened.push((host.clone(), cookie.clone()));
            }
        }
        flattened.sort_by(|a, b| a.0.cmp(&b.0));
        flattened
    }

    pub fn delete_cookie_at_index(&mut self, index: usize) {
        let flattened = self.get_flattened_cookies();
        if let Some((host, cookie_val)) = flattened.get(index) {
            if let Some(cookies) = self.cookie_jar.get_mut(host) {
                if let Some(pos) = cookies.iter().position(|c| c == cookie_val) {
                    cookies.remove(pos);
                    if cookies.is_empty() {
                        self.cookie_jar.remove(host);
                    }
                    self.save_cookies();
                }
            }
        }
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
        self.save_config();
    }

    pub fn process_url(&self) -> String {
        let mut final_url = self.active_tab().url.clone();
        let env = self.get_active_env();

        for (key, val) in &env.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            final_url = final_url.replace(&placeholder, val);
        }
        final_url
    }

    pub fn sync_url_to_params(&mut self) {
        let tab = self.active_tab_mut();
        if let Ok(u) = reqwest::Url::parse(&tab.url) {
            tab.params = u.query_pairs().into_owned().collect();
        } else {
            tab.params.clear();
        }
    }

    pub fn sync_params_to_url(&mut self) {
        let tab = self.active_tab_mut();
        if let Ok(mut u) = reqwest::Url::parse(&tab.url) {
            u.query_pairs_mut().clear().extend_pairs(&tab.params);
            tab.url = u.to_string();
        }
    }

    pub fn add_history(
        &mut self,
        method: String,
        url: String,
        duration: u128,
        status: u16,
        body: Option<String>,
        headers: std::collections::HashMap<String, String>,
        response_bytes: Option<Vec<u8>>,
        is_binary: bool,
    ) {
        let log = RequestLog {
            method,
            url,
            status,
            latency: duration,
            body,
            headers,
            response_bytes,
            is_binary,
        };
        self.request_history.insert(0, log);
        if self.request_history.len() > 50 {
            self.request_history.pop();
        }
        self.save_history();
    }

    fn load_history() -> Vec<RequestLog> {
        if let Ok(content) = std::fs::read_to_string("history.json") {
            if let Ok(history) = serde_json::from_str(&content) {
                return history;
            }
        }
        Vec::new()
    }

    fn save_history(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.request_history) {
            let _ = std::fs::write("history.json", json);
        }
    }

    pub fn toggle_diff_selection(&mut self, history_index: usize) {
        if let Some(base) = self.diff_base_index {
            if base == history_index {
                // Deselect if same
                self.diff_base_index = None;
                self.diff_target_index = None;
                self.show_diff_view = false;
            } else {
                // Set target and show diff
                self.diff_target_index = Some(history_index);
                self.show_diff_view = true;
                self.diff_list_state.select(Some(0));
            }
        } else {
            self.diff_base_index = Some(history_index);
        }
    }

    fn load_config() -> AppConfig {
        if let Ok(content) = std::fs::read_to_string("config.json") {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        AppConfig::default()
    }

    pub fn save_config(&self) {
        let config = AppConfig {
            theme_index: self.theme_index,
            selected_env_index: self.selected_env_index,
            zen_mode: self.zen_mode,
        };
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::write("config.json", json);
        }
    }

    fn load_cookies() -> std::collections::HashMap<String, Vec<String>> {
        if let Ok(content) = std::fs::read_to_string("cookies.json") {
             if let Ok(cookies) = serde_json::from_str(&content) {
                 return cookies;
             }
        }
        std::collections::HashMap::new()
    }

    fn save_cookies(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.cookie_jar) {
            let _ = std::fs::write("cookies.json", json);
        }
    }

    pub fn close_diff(&mut self) {
        self.show_diff_view = false;
        self.diff_list_state.select(None);
        self.diff_base_index = None;
        self.diff_target_index = None;
    }

    pub fn cycle_method(&mut self) {
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
        let tab = self.active_tab_mut();
        let current_pos = methods
            .iter()
            .position(|&m| m == tab.method)
            .unwrap_or(0);
        let next = (current_pos + 1) % methods.len();
        tab.method = methods[next].to_string();
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



    pub fn generate_docs(&mut self) {
        let md_res = crate::doc_gen::save_docs(&self.collections);
        let html_res = crate::doc_gen::save_html_docs(&self.collections);
        
        match (md_res, html_res) {
            (Ok(md_path), Ok(html_path)) => self.show_notification(format!("Docs Generated: {}, {}", md_path, html_path)),
            (Ok(md_path), Err(_)) => self.show_notification(format!("Docs Generated: {} (HTML failed)", md_path)),
            (Err(_), Ok(html_path)) => self.show_notification(format!("Docs Generated: {} (MD failed)", html_path)),
            (Err(e1), Err(e2)) => self.show_notification(format!("Docs Error: MD:{}, HTML:{}", e1, e2)),
        }
    }

    pub fn start_mock_server(&mut self) {
        if self.mock_server_running {
            return;
        }
        let handle = crate::mock_server::start_mock_server(self.mock_server_port, self.mock_routes.clone());
        self.mock_server_handle = Some(handle);
        self.mock_server_running = true;
        self.show_notification(format!("Mock Server Starting on port {}", self.mock_server_port));
    }

    pub fn stop_mock_server(&mut self) {
        if let Some(handle) = self.mock_server_handle.take() {
           handle.handle.abort();
        }
        self.mock_server_running = false;
        self.show_notification("Mock Server Stopped".to_string());
    }

    pub fn toggle_mock_server(&mut self) {
        if self.mock_server_running {
            self.stop_mock_server();
        } else {
            self.start_mock_server();
        }
    }

    pub fn restart_mock_server_if_running(&mut self) {
        if self.mock_server_running {
            self.stop_mock_server();
            self.start_mock_server();
        }
    }

    pub fn save_current_request(&mut self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let name = format!("Saved Request {}", timestamp);

        let tab = self.active_tab();
        let body_type_str = match tab.body_type {
            BodyType::Raw => "Raw",
            BodyType::FormData => "FormData",
            BodyType::GraphQL => "GraphQL",
            BodyType::Grpc => "Grpc",
        };

        if let Err(e) = Collection::save_to_file(
            &name,
            &tab.method,
            &tab.url,
            &tab.request_body,
            &tab.request_headers,
            &tab.extract_rules,
            &tab.form_data,
            body_type_str,
            &tab.graphql_query,
            &tab.graphql_variables,
            &tab.pre_request_script,
            &tab.post_request_script,
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
                    {
                        let tab = self.active_tab_mut();
                        tab.url = config.url;
                        tab.method = config.method;
                        tab.request_body = config.body.unwrap_or_default();
                        tab.request_headers = config.headers.unwrap_or_default();

                        tab.extract_rules = config
                            .extract
                            .map(|m| m.into_iter().collect())
                            .unwrap_or_default();
                        tab.form_data = config.form_data.unwrap_or_default();
                        tab.graphql_query = config.graphql_query.unwrap_or_default();
                        tab.graphql_variables = config.graphql_variables.unwrap_or_default();
                        tab.pre_request_script = config.pre_request_script.unwrap_or_default();
                        tab.post_request_script = config.post_request_script.unwrap_or_default();

                        tab.body_type = match config.body_type.as_deref() {
                            Some("FormData") => BodyType::FormData,
                            Some("GraphQL") => BodyType::GraphQL,
                            _ => BodyType::Raw,
                        };
                    }
                    self.sync_url_to_params();

                    let method = self.active_tab().method.clone();
                    let url = self.active_tab().url.clone();
                    self.show_notification(format!("Loaded: {} {}", method, url));
                }
            } else if idx > collection_count + 2 {
                let history_idx = idx - (collection_count + 3);
                if history_idx < self.request_history.len() {
                    if let Some(log) = self.request_history.get(history_idx).cloned() {
                        let tab = self.active_tab_mut();
                        tab.method = log.method.clone();
                        tab.url = log.url.clone();
                        tab.status_code = Some(log.status);
                        tab.latency = Some(log.latency);
                        
                        tab.response = log.body.clone();
                        tab.response_headers = log.headers.clone();
                        tab.response_bytes = log.response_bytes.clone();
                        tab.response_is_binary = log.is_binary;

                        if let Some(body_text) = &log.body {
                            if let Ok(val) = serde_json::from_str::<Value>(body_text) {
                                let root =
                                    crate::app::JsonEntry::from_value("root".to_string(), &val, 0);
                                tab.response_json = Some(vec![root]);
                            } else {
                                tab.response_json = None;
                            }
                        } else {
                            tab.response_json = None;
                        }

                        self.popup_message = Some("Restored from history".to_string());
                    }
                }
            }
        }
    }

    pub fn get_selected_history_index(&self) -> Option<usize> {
        if let Some(idx) = self.collection_state.selected() {
            let col_count = self.flattened_collection_only_count();
            // History items start at col_count + 3
            if idx > col_count + 2 {
                let hist_idx = idx - (col_count + 3);
                if hist_idx < self.request_history.len() {
                    return Some(hist_idx);
                }
            }
        }
        None
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

    pub fn guess_extension(&self) -> Option<String> {
        // Look at headers first
        // Note: Headers keys are lowercase in our HashMap (from network.rs)
        if let Some(ct) = self.active_tab().response_headers.get("content-type") {
            let ct = ct.to_lowercase();
            if ct.contains("json") { return Some("json".to_string()); }
            if ct.contains("html") { return Some("html".to_string()); }
            if ct.contains("png") { return Some("png".to_string()); }
            if ct.contains("jpeg") || ct.contains("jpg") { return Some("jpg".to_string()); }
            if ct.contains("pdf") { return Some("pdf".to_string()); }
            if ct.contains("xml") { return Some("xml".to_string()); }
            if ct.contains("javascript") { return Some("js".to_string()); }
            if ct.contains("text/plain") { return Some("txt".to_string()); }
        }
        None
    }

    pub fn download_response(&mut self) {
        if let Some(bytes) = &self.active_tab().response_bytes {
            // Try to find a good filename
            let mut filename = "response".to_string();
            
            // Check Content-Disposition header
            if let Some(cd) = self.active_tab().response_headers.get("content-disposition") {
                 if let Some(start) = cd.find("filename=") {
                     let rest = &cd[start + 9..];
                     let end = rest.find(';').unwrap_or(rest.len());
                     let name = rest[..end].trim_matches('"').to_string();
                     if !name.is_empty() {
                         filename = name;
                     }
                 }
            } else {
                // Use fallback with timestamp
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                filename = format!("response_{}", timestamp);
                 if let Some(ext) = self.guess_extension() {
                    filename = format!("{}.{}", filename, ext);
                 } else {
                     filename = format!("{}.bin", filename);
                 }
            }

            let mut path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            path.push(&filename);
            
            if std::fs::write(&path, bytes).is_ok() {
                self.show_notification(format!("Saved: {}", filename));
            } else {
                self.show_notification(format!("Failed to save {}", filename));
            }
        } else {
             self.show_notification("No response content to download".to_string());
        }
    }

    pub fn preview_response(&mut self) {
        if let Some(bytes) = &self.active_tab().response_bytes {
            let mut file_path = std::env::temp_dir();
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            let mut filename = format!("preview_{}", timestamp);
            
            if let Some(ext) = self.guess_extension() {
                filename = format!("{}.{}", filename, ext);
            } else {
                 filename = format!("{}.dat", filename);
            }
            file_path.push(filename);

            if std::fs::write(&file_path, bytes).is_ok() {
                 if webbrowser::open(file_path.to_str().unwrap()).is_ok() {
                      self.show_notification("Opened default viewer".to_string());
                 } else {
                      self.show_notification("Failed to open viewer".to_string());
                 }
            } else {
                 self.show_notification("Failed to write temp file".to_string());
            }
        } else {
            self.show_notification("No response content to preview".to_string());
        }
    }

    pub fn trigger_introspection(&mut self) {
        self.active_tab_mut().should_introspect_schema = true;
    }
    
    pub fn parse_schema_json(&mut self, json_str: &str) {
        // Simple manual parsing or use serde_json
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(types) = val.get("data")
                .and_then(|d| d.get("__schema"))
                .and_then(|s| s.get("types"))
                .and_then(|t| t.as_array()) 
            {
                let mut schema_types = Vec::new();
                for t in types {
                    if let Some(name) = t.get("name").and_then(|n| n.as_str()) {
                        if !name.starts_with("__") {
                            schema_types.push(name.to_string());
                        }
                    }
                }
                schema_types.sort();
                self.active_tab_mut().graphql_schema_types = schema_types;
                self.active_tab_mut().show_schema_modal = true; // Show modal with types
                self.show_notification("Schema Introspection Complete".to_string());
            } else {
                 self.show_notification("Invalid Schema Response".to_string());
            }
        } else {
             self.show_notification("Failed to parse Schema JSON".to_string());
        }
    }

    pub fn close_schema_modal(&mut self) {
        self.active_tab_mut().show_schema_modal = false;
    }

    pub fn generate_curl_command(&self) -> String {
        let tab = self.active_tab();
        let mut cmd = format!("curl -X {} \"{}\"", tab.method, self.process_url());

        match &tab.auth_type {
            AuthType::Bearer => {
                if !tab.auth_token.is_empty() {
                    cmd.push_str(&format!(
                        " -H \"Authorization: Bearer {}\"",
                        tab.auth_token
                    ));
                }
            }
            AuthType::Basic => {
                let creds = format!("{}:{}", tab.basic_auth_user, tab.basic_auth_pass);
                let _encoded = arboard::Clipboard::new()
                    .map(|_| "ENCODING_SKIPPED".to_string())
                    .unwrap_or("".to_string());
                cmd.push_str(&format!(" --user \"{}\"", creds));
            }
            AuthType::OAuth2 => {
                if !tab.auth_token.is_empty() {
                    cmd.push_str(&format!(
                        " -H \"Authorization: Bearer {}\"",
                        tab.auth_token
                    ));
                }
            }
            _ => {}
        }

        for (k, v) in &tab.request_headers {
            cmd.push_str(&format!(" -H \"{}: {}\"", k, v));
        }

        match tab.body_type {
            BodyType::Raw => {
                if !tab.request_body.is_empty() {
                    let escaped = tab.request_body.replace("'", "'\\''");
                    cmd.push_str(&format!(" -d '{}'", escaped));
                }
            }
            BodyType::FormData => {
                for (k, v, is_file) in &tab.form_data {
                    if *is_file {
                        cmd.push_str(&format!(" -F \"{} = @{}\"", k, v));
                    } else {
                        cmd.push_str(&format!(" -F \"{} = {}\"", k, v));
                    }
                }
            }
            BodyType::GraphQL => {
                let vars = if tab.graphql_variables.trim().is_empty() {
                    "{}"
                } else {
                    &tab.graphql_variables
                };
                let query = tab.graphql_query.replace("\n", " ").replace("'", "'\\''");
                let json_body = format!(r#"{{"query": "{}", "variables": {}}}"#, query, vars);
                 cmd.push_str(&format!(" -d '{}'", json_body));
            }
            BodyType::Grpc => {
                 cmd.push_str(" # gRPC not fully supported in CURL generator");
            }
        }

        cmd
    }

    /// Parse a curl command and populate the current request tab
    pub fn import_from_curl(&mut self, curl_cmd: &str) -> Result<(), String> {
        // Normalize the command: handle line continuations and clean up
        let cmd = curl_cmd
            .replace("\\\n", " ")
            .replace("\\\r\n", " ")
            .trim()
            .to_string();

        // Must start with 'curl'
        if !cmd.to_lowercase().starts_with("curl") {
            return Err("Command must start with 'curl'".to_string());
        }

        // Simple tokenizer that respects quotes
        let tokens = Self::tokenize_curl(&cmd)?;
        
        let mut url = String::new();
        let mut method = "GET".to_string();
        let mut headers: Vec<(String, String)> = Vec::new();
        let mut body = String::new();
        let mut form_data: Vec<(String, String, bool)> = Vec::new();
        let mut auth_user = String::new();
        let mut auth_pass = String::new();

        let mut i = 1; // Skip 'curl'
        while i < tokens.len() {
            let token = &tokens[i];
            match token.as_str() {
                "-X" | "--request" => {
                    if i + 1 < tokens.len() {
                        method = tokens[i + 1].to_uppercase();
                        i += 1;
                    }
                }
                "-H" | "--header" => {
                    if i + 1 < tokens.len() {
                        let header = &tokens[i + 1];
                        if let Some(colon_pos) = header.find(':') {
                            let key = header[..colon_pos].trim().to_string();
                            let value = header[colon_pos + 1..].trim().to_string();
                            headers.push((key, value));
                        }
                        i += 1;
                    }
                }
                "-d" | "--data" | "--data-raw" | "--data-binary" => {
                    if i + 1 < tokens.len() {
                        body = tokens[i + 1].clone();
                        if method == "GET" {
                            method = "POST".to_string();
                        }
                        i += 1;
                    }
                }
                "-F" | "--form" => {
                    if i + 1 < tokens.len() {
                        let form_item = &tokens[i + 1];
                        if let Some(eq_pos) = form_item.find('=') {
                            let key = form_item[..eq_pos].trim().to_string();
                            let value = form_item[eq_pos + 1..].trim().to_string();
                            let is_file = value.starts_with('@');
                            let clean_value = if is_file { value[1..].to_string() } else { value };
                            form_data.push((key, clean_value, is_file));
                        }
                        if method == "GET" {
                            method = "POST".to_string();
                        }
                        i += 1;
                    }
                }
                "-u" | "--user" => {
                    if i + 1 < tokens.len() {
                        let auth = &tokens[i + 1];
                        if let Some(colon_pos) = auth.find(':') {
                            auth_user = auth[..colon_pos].to_string();
                            auth_pass = auth[colon_pos + 1..].to_string();
                        }
                        i += 1;
                    }
                }
                "-A" | "--user-agent" => {
                    if i + 1 < tokens.len() {
                        headers.push(("User-Agent".to_string(), tokens[i + 1].clone()));
                        i += 1;
                    }
                }
                "-b" | "--cookie" => {
                    if i + 1 < tokens.len() {
                        headers.push(("Cookie".to_string(), tokens[i + 1].clone()));
                        i += 1;
                    }
                }
                "-e" | "--referer" => {
                    if i + 1 < tokens.len() {
                        headers.push(("Referer".to_string(), tokens[i + 1].clone()));
                        i += 1;
                    }
                }
                // Skip flags we don't care about
                "-k" | "--insecure" | "-v" | "--verbose" | "-s" | "--silent" 
                | "-L" | "--location" | "-i" | "--include" | "--compressed" => {}
                // Skip flags with arguments we ignore
                "-o" | "--output" | "--connect-timeout" | "-m" | "--max-time" => {
                    i += 1; // Skip the argument too
                }
                _ => {
                    // If it looks like a URL
                    if token.starts_with("http://") || token.starts_with("https://") 
                        || token.starts_with("ws://") || token.starts_with("wss://") {
                        url = token.clone();
                    } else if !token.starts_with('-') && url.is_empty() {
                        // Might be a URL without protocol
                        url = token.clone();
                    }
                }
            }
            i += 1;
        }

        if url.is_empty() {
            return Err("No URL found in curl command".to_string());
        }

        // Populate the current tab
        let tab = self.active_tab_mut();
        tab.url = url;
        tab.method = method;
        tab.request_headers = headers.into_iter().collect();
        
        if !form_data.is_empty() {
            tab.body_type = BodyType::FormData;
            tab.form_data = form_data;
        } else if !body.is_empty() {
            tab.body_type = BodyType::Raw;
            tab.request_body = body;
        }

        if !auth_user.is_empty() {
            tab.auth_type = AuthType::Basic;
            tab.basic_auth_user = auth_user;
            tab.basic_auth_pass = auth_pass;
        }

        // Sync URL to params
        self.sync_url_to_params();

        Ok(())
    }

    /// Tokenize a curl command respecting quoted strings
    fn tokenize_curl(cmd: &str) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;

        for ch in cmd.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if !in_single_quote => {
                    escape_next = true;
                }
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                }
                ' ' | '\t' if !in_single_quote && !in_double_quote => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        if in_single_quote || in_double_quote {
            return Err("Unclosed quote in curl command".to_string());
        }

        Ok(tokens)
    }

    pub fn generate_python_code(&self) -> String {
        let tab = self.active_tab();
        let mut code = String::from("import requests\n\n");
        code.push_str(&format!("url = \"{}\"\n", self.process_url()));

        code.push_str("headers = {\n");
        for (k, v) in &tab.request_headers {
            code.push_str(&format!("    \"{}\": \"{}\",\n", k, v));
        }
        if tab.auth_type == AuthType::Bearer || tab.auth_type == AuthType::OAuth2 {
            if !tab.auth_token.is_empty() {
                code.push_str(&format!(
                    "    \"Authorization\": \"Bearer {}\",\n",
                    tab.auth_token
                ));
            }
        }
        code.push_str("}\n\n");

        match tab.body_type {
            BodyType::Raw => {
                if !tab.request_body.is_empty() {
                    code.push_str(&format!("payload = '''{}'''\n\n", tab.request_body));
                    code.push_str(&format!(
                        "response = requests.request(\"{}\", url, headers=headers, data=payload)",
                        tab.method
                    ));
                } else {
                    code.push_str(&format!(
                        "response = requests.request(\"{}\", url, headers=headers)",
                        tab.method
                    ));
                }
            }
            BodyType::FormData => {
                code.push_str("files = [\n");
                for (k, v, is_file) in &tab.form_data {
                    if *is_file {
                        code.push_str(&format!("    ('{}', open('{}', 'rb')),\n", k, v));
                    } else {
                        code.push_str(&format!("    ('{}', (None, '{}')),\n", k, v));
                    }
                }
                code.push_str("]\n\n");
                code.push_str(&format!(
                    "response = requests.request(\"{}\", url, headers=headers, files=files)",
                    tab.method
                ));
            }
            _ => {
                code.push_str(&format!(
                    "response = requests.request(\"{}\", url, headers=headers)",
                    tab.method
                ));
            }
        }

        code.push_str("\n\nprint(response.text)");
        code
    }

    pub fn generate_javascript_code(&self) -> String {
        let tab = self.active_tab();
        let mut code = format!(
            "const url = \"{}\";\nconst options = {{\n  method: '{}',\n  headers: {{\n",
            self.process_url(),
            tab.method
        );

        for (k, v) in &tab.request_headers {
            code.push_str(&format!("    '{}': '{}',\n", k, v));
        }
        if tab.auth_type == AuthType::Bearer || tab.auth_type == AuthType::OAuth2 {
            if !tab.auth_token.is_empty() {
                code.push_str(&format!(
                    "    'Authorization': 'Bearer {}',\n",
                    tab.auth_token
                ));
            }
        }
        code.push_str("  },\n");

        if tab.body_type == BodyType::Raw && !tab.request_body.is_empty() {
            code.push_str(&format!("  body: JSON.stringify({})\n", tab.request_body));
        } else if tab.body_type == BodyType::FormData {
            code.push_str("  body: formData\n");
        }

        code.push_str("};\n\n");

        if tab.body_type == BodyType::FormData {
            code.push_str("// Note: Construct FormData manually if needed\n\n");
        }

        code.push_str("try {\n  const response = await fetch(url, options);\n  const data = await response.json();\n  console.log(data);\n} catch (error) {\n  console.error(error);\n}");
        code
    }


    pub fn generate_go_code(&self) -> String {
        let tab = self.active_tab();
        let mut code = String::from("package main\n\nimport (\n\t\"fmt\"\n\t\"net/http\"\n\t\"io/ioutil\"\n");

        if tab.body_type == BodyType::Raw && !tab.request_body.is_empty() {
            code.push_str("\t\"strings\"\n");
        }
        if tab.body_type == BodyType::FormData {
            code.push_str("\t\"bytes\"\n\t\"mime/multipart\"\n\t\"os\"\n\t\"io\"\n\t\"path/filepath\"\n");
        }
        code.push_str(")\n\nfunc main() {\n");
        code.push_str(&format!("\turl := \"{}\"\n", self.process_url()));
        code.push_str(&format!("\tmethod := \"{}\"\n", tab.method));
        
        if tab.body_type == BodyType::Raw && !tab.request_body.is_empty() {
             let safe_body = tab.request_body.replace("`", "` + \"`\" + `");
             code.push_str(&format!("\tpayload := strings.NewReader(`{}`)\n", safe_body));
             code.push_str("\n\tclient := &http.Client{}\n");
             code.push_str("\treq, err := http.NewRequest(method, url, payload)\n");
        } else if tab.body_type == BodyType::FormData {
             code.push_str("\tpayload := &bytes.Buffer{}\n");
             code.push_str("\twriter := multipart.NewWriter(payload)\n");
             for (k, v, is_file) in &tab.form_data {
                 if *is_file {
                      code.push_str(&format!("\tfile, err := os.Open(\"{}\")\n", v));
                      code.push_str("\tif err != nil {\n\t\tfmt.Println(err)\n\t\treturn\n\t}\n\tdefer file.Close()\n");
                      code.push_str(&format!("\tpart, err := writer.CreateFormFile(\"{}\", filepath.Base(\"{}\"))\n", k, v));
                      code.push_str("\t_, err = io.Copy(part, file)\n");
                 } else {
                      code.push_str(&format!("\t_ = writer.WriteField(\"{}\", \"{}\")\n", k, v));
                 }
             }
             code.push_str("\terr := writer.Close()\n");
             code.push_str("\tif err != nil {\n\t\tfmt.Println(err)\n\t\treturn\n\t}\n");
             
             code.push_str("\n\tclient := &http.Client{}\n");
             code.push_str("\treq, err := http.NewRequest(method, url, payload)\n");
             code.push_str("\treq.Header.Set(\"Content-Type\", writer.FormDataContentType())\n");

        } else {
             code.push_str("\n\tclient := &http.Client{}\n");
             code.push_str("\treq, err := http.NewRequest(method, url, nil)\n");
        }

        code.push_str("\tif err != nil {\n\t\tfmt.Println(err)\n\t\treturn\n\t}\n");

        for (k, v) in &tab.request_headers {
            code.push_str(&format!("\treq.Header.Add(\"{}\", \"{}\")\n", k, v));
        }

        if tab.auth_type == AuthType::Bearer || tab.auth_type == AuthType::OAuth2 {
             if !tab.auth_token.is_empty() {
                 code.push_str(&format!("\treq.Header.Add(\"Authorization\", \"Bearer {}\")\n", tab.auth_token));
             }
        }

        code.push_str("\n\tres, err := client.Do(req)\n");
        code.push_str("\tif err != nil {\n\t\tfmt.Println(err)\n\t\treturn\n\t}\n");
        code.push_str("\tdefer res.Body.Close()\n\n");
        code.push_str("\tbody, err := ioutil.ReadAll(res.Body)\n");
        code.push_str("\tif err != nil {\n\t\tfmt.Println(err)\n\t\treturn\n\t}\n");
        code.push_str("\tfmt.Println(string(body))\n}\n");

        code
    }

    pub fn generate_rust_code(&self) -> String {
        let tab = self.active_tab();
        let mut code = String::from("#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
        code.push_str("\tlet client = reqwest::Client::new();\n");
        
        if tab.body_type == BodyType::FormData {
            code.push_str("\tlet form = reqwest::multipart::Form::new()\n");
            for (k, v, is_file) in &tab.form_data {
                if *is_file {
                     code.push_str(&format!("\t\t.file(\"{}\", \"{}\").await?\n", k, v));
                } else {
                     code.push_str(&format!("\t\t.text(\"{}\", \"{}\")\n", k, v));
                }
            }
            code.push_str("\t\t;\n");
        }

        code.push_str(&format!("\tlet res = client.request(reqwest::Method::{}, \"{}\")\n", tab.method.to_uppercase(), self.process_url()));

        for (k, v) in &tab.request_headers {
            code.push_str(&format!("\t\t.header(\"{}\", \"{}\")\n", k, v));
        }
        
        if tab.auth_type == AuthType::Bearer || tab.auth_type == AuthType::OAuth2 {
            if !tab.auth_token.is_empty() {
                code.push_str(&format!("\t\t.bearer_auth(\"{}\")\n", tab.auth_token));
            }
        }

        if tab.body_type == BodyType::Raw && !tab.request_body.is_empty() {
             let safe_body = tab.request_body.replace("\"", "\\\"");
             code.push_str(&format!("\t\t.body(\"{}\")\n", safe_body));
        } else if tab.body_type == BodyType::FormData {
             code.push_str("\t\t.multipart(form)\n");
        }

        code.push_str("\t\t.send()\n\t\t.await?;\n");
        code.push_str("\tprintln!(\"{}\", res.text().await?);\n");
        code.push_str("\tOk(())\n}\n");
        code
    }

    pub fn generate_ruby_code(&self) -> String {
        let tab = self.active_tab();
        let mut code = String::from("require 'uri'\nrequire 'net/http'\n\n");
        code.push_str(&format!("url = URI(\"{}\")\n\n", self.process_url()));
        code.push_str("http = Net::HTTP.new(url.host, url.port)\n");
        code.push_str("http.use_ssl = true\n\n");
        
        let method_lower = tab.method.to_lowercase();
        let method_start = method_lower.chars().next().unwrap_or('g').to_uppercase();
        let method_rest = if method_lower.len() > 1 { &method_lower[1..] } else { "" };
        let method_class = format!("{}{}", method_start, method_rest);
        
        code.push_str(&format!("request = Net::HTTP::{}.new(url)\n", method_class));

        for (k, v) in &tab.request_headers {
             code.push_str(&format!("request[\"{}\"] = \"{}\"\n", k, v));
        }
        
        if tab.auth_type == AuthType::Bearer || tab.auth_type == AuthType::OAuth2 {
            if !tab.auth_token.is_empty() {
                 code.push_str(&format!("request[\"Authorization\"] = \"Bearer {}\"\n", tab.auth_token));
            }
        }

        if tab.body_type == BodyType::Raw && !tab.request_body.is_empty() {
            let safe_body = tab.request_body.replace("\"", "\\\"");
            code.push_str(&format!("request.body = \"{}\"\n", safe_body));
        } else if tab.body_type == BodyType::FormData {
            code.push_str("boundary = \"PostDadBoundary\"\n");
            code.push_str("request[\"Content-Type\"] = \"multipart/form-data; boundary=#{boundary}\"\n");
            code.push_str("body = []\n");
            for (k, v, is_file) in &tab.form_data {
                if *is_file {
                    code.push_str(&format!("body << \"--#{{boundary}}\\r\\n\"\n"));
                    code.push_str(&format!("body << \"Content-Disposition: form-data; name=\\\"{}\\\"; filename=\\\"{}\\\"\\r\\n\"\n", k, v));
                    code.push_str("body << \"Content-Type: application/octet-stream\\r\\n\\r\\n\"\n");
                    code.push_str(&format!("body << File.read(\"{}\")\n", v));
                    code.push_str("body << \"\\r\\n\"\n");
                } else {
                    code.push_str(&format!("body << \"--#{{boundary}}\\r\\n\"\n"));
                    code.push_str(&format!("body << \"Content-Disposition: form-data; name=\\\"{}\\\";\\r\\n\\r\\n\"\n", k));
                    code.push_str(&format!("body << \"{}\\r\\n\"\n", v));
                }
            }
            code.push_str("body << \"--#{boundary}--\\r\\n\"\n");
            code.push_str("request.body = body.join\n");
        }

        code.push_str("\nresponse = http.request(request)\n");
        code.push_str("puts response.read_body\n");
        code
    }

    pub fn generate_php_code(&self) -> String {
         let tab = self.active_tab();
         let mut code = String::from("<?php\n\n$curl = curl_init();\n\ncurl_setopt_array($curl, array(\n");
         code.push_str(&format!("  CURLOPT_URL => '{}',\n", self.process_url()));
         code.push_str("  CURLOPT_RETURNTRANSFER => true,\n  CURLOPT_ENCODING => '',\n  CURLOPT_MAXREDIRS => 10,\n  CURLOPT_TIMEOUT => 0,\n  CURLOPT_FOLLOWLOCATION => true,\n  CURLOPT_HTTP_VERSION => CURL_HTTP_VERSION_1_1,\n");
         code.push_str(&format!("  CURLOPT_CUSTOMREQUEST => '{}',\n", tab.method));
         
         if tab.body_type == BodyType::Raw && !tab.request_body.is_empty() {
             let safe_body = tab.request_body.replace("'", "\\'");
             code.push_str(&format!("  CURLOPT_POSTFIELDS => '{}',\n", safe_body));
         } else if tab.body_type == BodyType::FormData {
             code.push_str("  CURLOPT_POSTFIELDS => array(\n");
             for (k, v, is_file) in &tab.form_data {
                  if *is_file {
                       code.push_str(&format!("    '{}' => new CURLFile('{}'),\n", k, v));
                  } else {
                       code.push_str(&format!("    '{}' => '{}',\n", k, v));
                  }
             }
             code.push_str("  ),\n");
         }

         code.push_str("  CURLOPT_HTTPHEADER => array(\n");
         for (k, v) in &tab.request_headers {
              code.push_str(&format!("    '{}: {}',\n", k, v));
         }
         if tab.auth_type == AuthType::Bearer || tab.auth_type == AuthType::OAuth2 {
             if !tab.auth_token.is_empty() {
                 code.push_str(&format!("    'Authorization: Bearer {}',\n", tab.auth_token));
             }
         }
         code.push_str("  ),\n));\n\n$response = curl_exec($curl);\n\ncurl_close($curl);\necho $response;\n");
         code
    }

      pub fn generate_csharp_code(&self) -> String {
        let tab = self.active_tab();
        let mut code = String::from("var client = new HttpClient();\n");
        let method_start = tab.method.chars().next().unwrap_or('G').to_uppercase();
        let method_rest = if tab.method.len() > 1 { tab.method[1..].to_lowercase() } else { String::new() };
        let method = format!("{}{}", method_start, method_rest);
        code.push_str(&format!("var request = new HttpRequestMessage(HttpMethod.{}, \"{}\");\n", method, self.process_url()));

        for (k, v) in &tab.request_headers {
            code.push_str(&format!("request.Headers.Add(\"{}\", \"{}\");\n", k, v));
        }
        
        if tab.auth_type == AuthType::Bearer || tab.auth_type == AuthType::OAuth2 {
            if !tab.auth_token.is_empty() {
                code.push_str(&format!("request.Headers.Add(\"Authorization\", \"Bearer {}\");\n", tab.auth_token));
            }
        }

        if tab.body_type == BodyType::Raw && !tab.request_body.is_empty() {
             let safe_body = tab.request_body.replace("\"", "\\\"");
             code.push_str(&format!("var content = new StringContent(\"{}\", null, \"application/json\");\n", safe_body));
             code.push_str("request.Content = content;\n");
        } else if tab.body_type == BodyType::FormData {
            code.push_str("var content = new MultipartFormDataContent();\n");
            for (k, v, is_file) in &tab.form_data {
                if *is_file {
                     code.push_str(&format!("content.Add(new ByteArrayContent(File.ReadAllBytes(\"{}\")), \"{}\", \"{}\");\n", v, k, v));
                } else {
                     code.push_str(&format!("content.Add(new StringContent(\"{}\"), \"{}\");\n", v, k));
                }
            }
            code.push_str("request.Content = content;\n");
        }

        code.push_str("var response = await client.SendAsync(request);\n");
        code.push_str("response.EnsureSuccessStatusCode();\n");
        code.push_str("Console.WriteLine(await response.Content.ReadAsStringAsync());\n");
        
        code
    }

    pub fn copy_to_clipboard(&mut self, text: String) {
        if self.clipboard.is_none() {
             // Try to re-initialize if it failed initially
             self.clipboard = Clipboard::new().ok();
        }

        if let Some(clipboard) = &mut self.clipboard {
            if let Err(e) = clipboard.set_text(text) {
                self.popup_message = Some(format!("Clipboard Error: {}", e));
            } else {
                 self.popup_message = Some("Copied to clipboard!".to_string());
            }
        } else {
             self.popup_message = Some("Clipboard unavailable".to_string());
        }
    }

    pub fn copy_response(&mut self) {
        let tab = self.active_tab();
        if tab.response_is_binary {
            self.popup_message = Some("Cannot copy binary response to clipboard".to_string());
            return;
        }
        
        if let Some(ref response) = tab.response {
            let text = response.clone();
            self.copy_to_clipboard(text);
            self.popup_message = Some("Response copied to clipboard!".to_string());
        } else {
            self.popup_message = Some("No response to copy".to_string());
        }
    }

    pub fn toggle_current_selection(&mut self) {
        let tab = self.active_tab_mut();
        if let Some(selected_idx) = tab.json_list_state.selected() {
            if let Some(entries) = &mut tab.response_json {
                let mut current_idx = selected_idx;
                if let Some(node) = Self::get_mut_node_at_index(entries, &mut current_idx) {
                    node.is_expanded = !node.is_expanded;
                }
            }
        }
    }

    pub fn set_expanded_current_selection(&mut self, expanded: bool) {
        let tab = self.active_tab_mut();
        if let Some(selected_idx) = tab.json_list_state.selected() {
            if let Some(entries) = &mut tab.response_json {
                let mut current_idx = selected_idx;
                if let Some(node) = Self::get_mut_node_at_index(entries, &mut current_idx) {
                    node.is_expanded = expanded;
                }
            }
        }
    }

    pub fn duplicate_tab(&mut self) {
        let mut new_tab = self.active_tab().clone();
        new_tab.name = format!("{} (Copy)", new_tab.name);
        
        // Reset response state for the new tab
        new_tab.response = None;
        new_tab.response_bytes = None;
        new_tab.response_is_binary = false;
        new_tab.response_image = None;
        new_tab.response_json = None;
        new_tab.response_headers = std::collections::HashMap::new();
        new_tab.status_code = None;
        new_tab.latency = None;
        new_tab.latency_history = Vec::new();
        new_tab.is_loading = false;
        new_tab.test_results = Vec::new();
        new_tab.script_output = Vec::new();
        new_tab.ws_connected = false;
        new_tab.ws_messages = Vec::new();

        self.tabs.push(new_tab);
        self.active_tab = self.tabs.len() - 1;
        self.show_notification("Tab Duplicated".to_string());
    }

    pub fn clear_history(&mut self) {
        self.request_history.clear();
        self.save_history();
        self.show_notification("Request History Cleared".to_string());
    }

    pub fn clear_cookies(&mut self) {
        self.cookie_jar.clear();
        self.save_cookies();
        self.show_notification("Cookies Cleared".to_string());
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
        let tab = self.active_tab();
        if let Some(entries) = &tab.response_json {
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
        self.active_tab_mut().response_scroll.0 += 1;
    }

    pub fn scroll_up(&mut self) {
        let tab = self.active_tab_mut();
        if tab.response_scroll.0 > 0 {
            tab.response_scroll.0 -= 1;
        }
    }

    pub fn scroll_page_down(&mut self) {
        let tab = self.active_tab_mut();
        if let Some(entries) = &tab.response_json {
            let count = Self::count_visible(entries);
            if count > 0 {
                let current = tab.json_list_state.selected().unwrap_or(0);
                let next = (current + 10).min(count - 1);
                tab.json_list_state.select(Some(next));
                return;
            }
        }
        tab.response_scroll.0 += 10;
    }

    pub fn scroll_page_up(&mut self) {
        let tab = self.active_tab_mut();
        if let Some(entries) = &tab.response_json {
            let count = Self::count_visible(entries);
            if count > 0 {
                let current = tab.json_list_state.selected().unwrap_or(0);
                let next = if current > 10 { current - 10 } else { 0 };
                tab.json_list_state.select(Some(next));
                return;
            }
        }
        if tab.response_scroll.0 > 10 {
            tab.response_scroll.0 -= 10;
        } else {
            tab.response_scroll.0 = 0;
        }
    }

    pub fn next_item(&mut self) {
        let count = self.calculate_visible_item_count();
        if count == 0 {
            self.scroll_down();
            return;
        }

        let tab = self.active_tab_mut();
        let i = match tab.json_list_state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        tab.json_list_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let count = self.calculate_visible_item_count();
        if count == 0 {
            self.scroll_up();
            return;
        }

        let tab = self.active_tab_mut();
        let i = match tab.json_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        tab.json_list_state.select(Some(i));
    }
}

// Commands for Command Palette
#[derive(Clone, Debug, PartialEq)]
pub struct CommandAction {
    pub name: &'static str,
    pub desc: &'static str,
}

pub fn get_available_commands() -> Vec<CommandAction> {
    vec![
        CommandAction { name: "New Tab", desc: "Open a new request tab" },
        CommandAction { name: "Duplicate Tab", desc: "Duplicate current tab" },
        CommandAction { name: "Close Tab", desc: "Close current tab" },
        CommandAction { name: "Next Tab", desc: "Switch to next tab" },
        CommandAction { name: "Prev Tab", desc: "Switch to previous tab" },
        CommandAction { name: "Toggle Sidebar", desc: "Show/Hide Sidebar" },
        CommandAction { name: "Toggle Zen Mode", desc: "Show/Hide UI Chrome" },
        CommandAction { name: "Switch Theme", desc: "Rotate through themes" },
        CommandAction { name: "Toggle WebSocket", desc: "Switch between HTTP/WebSocket" },
        CommandAction { name: "Filter Collections", desc: "Search/Filter sidebar" },
        CommandAction { name: "Clear History", desc: "Clear request history" },
        CommandAction { name: "Clear Cookies", desc: "Clear all saved cookies" },
        CommandAction { name: "Manage Cookies", desc: "View and delete cookies" },
        CommandAction { name: "Export HTML Docs", desc: "Generate API_DOCS.html" },
        CommandAction { name: "Help", desc: "Show keyboard shortcuts" },
        CommandAction { name: "Quit", desc: "Exit Application" },
    ]
}
