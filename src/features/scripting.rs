use rhai::{Engine, Scope};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Result of running a pre-request script
#[derive(Debug, Clone, Default)]
pub struct ScriptResult {
    pub headers: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub body_override: Option<String>,
    pub url_override: Option<String>,
    pub errors: Vec<String>,
}

pub fn run_script(
    script: &str,
    method: &str,
    url: &str,
    current_headers: &HashMap<String, String>,
    current_body: &str,
    env_vars: &HashMap<String, String>,
) -> ScriptResult {
    if script.trim().is_empty() {
        return ScriptResult::default();
    }

    let mut engine = Engine::new();

    // Shared state for the script to modify
    let headers: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(current_headers.clone()));
    let variables: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(env_vars.clone()));
    let body_override: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let url_override: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let logs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Clone for closures
    let headers_set = headers.clone();
    let headers_get = headers.clone();
    let vars_set = variables.clone();
    let vars_get = variables.clone();
    let body_set = body_override.clone();
    let url_set = url_override.clone();
    let logs_clone = logs.clone();

    // Register set_header
    engine.register_fn("set_header", move |name: &str, value: &str| {
        if let Ok(mut h) = headers_set.lock() {
            h.insert(name.to_string(), value.to_string());
        }
    });

    // Register get_header
    engine.register_fn("get_header", move |name: &str| -> String {
        if let Ok(h) = headers_get.lock() {
            h.get(name).cloned().unwrap_or_default()
        } else {
            String::new()
        }
    });

    // Register set_var
    engine.register_fn("set_var", move |name: &str, value: &str| {
        if let Ok(mut v) = vars_set.lock() {
            v.insert(name.to_string(), value.to_string());
        }
    });

    // Register get_var
    engine.register_fn("get_var", move |name: &str| -> String {
        if let Ok(v) = vars_get.lock() {
            v.get(name).cloned().unwrap_or_default()
        } else {
            String::new()
        }
    });

    // Register set_body
    engine.register_fn("set_body", move |body: &str| {
        if let Ok(mut b) = body_set.lock() {
            *b = Some(body.to_string());
        }
    });

    // Register set_url
    engine.register_fn("set_url", move |url: &str| {
        if let Ok(mut u) = url_set.lock() {
            *u = Some(url.to_string());
        }
    });

    // Register timestamp
    engine.register_fn("timestamp", || -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    });

    // Register timestamp_ms
    engine.register_fn("timestamp_ms", || -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    });

    // Register uuid
    engine.register_fn("uuid", || -> String {
        // Simple UUID v4 generation using random bytes
        use rand::Rng;
        let mut rng = rand::rng();
        let bytes: [u8; 16] = rng.random();
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            (bytes[6] & 0x0f) | 0x40, bytes[7],
            (bytes[8] & 0x3f) | 0x80, bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    });

    // Register base64_encode
    engine.register_fn("base64_encode", |text: &str| -> String {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut encoder = Base64Encoder::new(&mut buf);
            let _ = encoder.write_all(text.as_bytes());
            let _ = encoder.finish();
        }
        String::from_utf8(buf).unwrap_or_default()
    });

    // Register base64_decode
    engine.register_fn("base64_decode", |text: &str| -> String {
        base64_decode_str(text).unwrap_or_default()
    });

    // Register print for debugging
    engine.register_fn("print", move |msg: &str| {
        if let Ok(mut l) = logs_clone.lock() {
            l.push(msg.to_string());
        }
    });

    // Create scope with request context
    let mut scope = Scope::new();
    scope.push_constant("METHOD", method.to_string());
    scope.push_constant("URL", url.to_string());
    scope.push_constant("BODY", current_body.to_string());

    // Run the script
    let mut result = ScriptResult::default();

    match engine.compile(script) {
        Ok(ast) => {
            if let Err(e) = engine.run_ast_with_scope(&mut scope, &ast) {
                result.errors.push(format!("Script error: {}", e));
            }
        }
        Err(e) => {
            result.errors.push(format!("Script compile error: {}", e));
        }
    }

    // Collect results
    if let Ok(h) = headers.lock() {
        result.headers = h.clone();
    }
    if let Ok(v) = variables.lock() {
        result.variables = v.clone();
    }
    if let Ok(b) = body_override.lock() {
        result.body_override = b.clone();
    }
    if let Ok(u) = url_override.lock() {
        result.url_override = u.clone();
    }
    if let Ok(l) = logs.lock() {
        for log in l.iter() {
            result.errors.push(format!("[LOG] {}", log));
        }
    }

    result
}

// Simple Base64 encoder
struct Base64Encoder<'a> {
    writer: &'a mut Vec<u8>,
    buffer: [u8; 3],
    buffer_len: usize,
}

impl<'a> Base64Encoder<'a> {
    fn new(writer: &'a mut Vec<u8>) -> Self {
        Self {
            writer,
            buffer: [0; 3],
            buffer_len: 0,
        }
    }

    fn finish(mut self) -> std::io::Result<()> {
        if self.buffer_len > 0 {
            self.encode_block();
        }
        Ok(())
    }

    fn encode_block(&mut self) {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let b0 = self.buffer[0] as usize;
        let b1 = self.buffer[1] as usize;
        let b2 = self.buffer[2] as usize;

        self.writer.push(ALPHABET[b0 >> 2]);
        self.writer.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)]);

        if self.buffer_len > 1 {
            self.writer.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)]);
        } else {
            self.writer.push(b'=');
        }

        if self.buffer_len > 2 {
            self.writer.push(ALPHABET[b2 & 0x3f]);
        } else {
            self.writer.push(b'=');
        }

        self.buffer = [0; 3];
        self.buffer_len = 0;
    }
}

impl<'a> std::io::Write for Base64Encoder<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            self.buffer[self.buffer_len] = byte;
            self.buffer_len += 1;
            if self.buffer_len == 3 {
                self.encode_block();
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn base64_decode_str(input: &str) -> Option<String> {
    const DECODE_TABLE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1,
        -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4,
        5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1,
        -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];

    let input = input.trim_end_matches('=');
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for c in input.chars() {
        let val = if (c as usize) < 128 {
            DECODE_TABLE[c as usize]
        } else {
            -1
        };
        if val < 0 {
            continue;
        }
        buffer = (buffer << 6) | (val as u32);
        bits_collected += 6;
        if bits_collected >= 8 {
            bits_collected -= 8;
            output.push(((buffer >> bits_collected) & 0xff) as u8);
        }
    }

    String::from_utf8(output).ok()
}

#[derive(Debug, Clone, Default)]
pub struct PostScriptResult {
    pub tests: Vec<(String, bool)>,
    pub errors: Vec<String>,
}

pub fn run_post_script(
    script: &str,
    status: u16,
    body: &str,
    headers: &HashMap<String, String>,
    latency: u128,
) -> PostScriptResult {
    if script.trim().is_empty() {
        return PostScriptResult::default();
    }

    let mut engine = Engine::new();

    // Shared state
    let tests: Arc<Mutex<Vec<(String, bool)>>> = Arc::new(Mutex::new(Vec::new()));
    let logs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let tests_clone = tests.clone();
    let logs_clone = logs.clone();

    // Capture response data for closures
    let headers_arc = Arc::new(headers.clone());
    let body_string = body.to_string();

    // Register test function
    // Usage: test("Status is 200", status_code() == 200);
    engine.register_fn("test", move |name: &str, result: bool| {
        if let Ok(mut t) = tests_clone.lock() {
            t.push((name.to_string(), result));
        }
    });

    // Register status_code
    engine.register_fn("status_code", move || -> i64 { status as i64 });

    // Register response_time (latency)
    engine.register_fn("response_time", move || -> i64 { latency as i64 });

    // Register response_body
    let body_str_clone = body_string.clone();
    engine.register_fn("response_body", move || -> String {
        body_str_clone.clone()
    });

    // Register get_header
    let headers_clone = headers_arc.clone();
    engine.register_fn("get_header", move |name: &str| -> String {
        headers_clone.get(name).cloned().unwrap_or_default()
    });

    // Register json_path
    let body_json = body_string.clone();
    engine.register_fn("json_path", move |query: &str| -> String {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_json) {
            let mut selector = jsonpath_lib::selector(&json);
            if let Ok(matches) = selector(query)
                && let Some(first) = matches.first() {
                    // Return raw string or json string?
                    if let Some(s) = first.as_str() {
                        return s.to_string();
                    } else {
                        return first.to_string();
                    }
                }
        }
        String::new()
    });

    // Register print
    engine.register_fn("print", move |msg: &str| {
        if let Ok(mut l) = logs_clone.lock() {
            l.push(msg.to_string());
        }
    });

    let mut result = PostScriptResult::default();

    match engine.eval::<()>(script) {
        Ok(_) => {}
        Err(e) => {
            result.errors.push(format!("Script error: {}", e));
        }
    }

    if let Ok(t) = tests.lock() {
        result.tests = t.clone();
    }
    if let Ok(l) = logs.lock() {
        for log in l.iter() {
            result.errors.push(format!("[LOG] {}", log));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_script() {
        let result = run_script(
            r#"
            set_header("X-Custom", "test");
            set_var("my_var", "hello");
            "#,
            "GET",
            "https://example.com",
            &HashMap::new(),
            "",
            &HashMap::new(),
        );

        assert_eq!(result.headers.get("X-Custom"), Some(&"test".to_string()));
        assert_eq!(result.variables.get("my_var"), Some(&"hello".to_string()));
    }

    #[test]
    fn test_timestamp() {
        let result = run_script(
            r#"
            let ts = timestamp();
            set_header("X-Timestamp", ts.to_string());
            "#,
            "GET",
            "https://example.com",
            &HashMap::new(),
            "",
            &HashMap::new(),
        );

        assert!(result.headers.contains_key("X-Timestamp"));
    }

    #[test]
    fn test_post_script() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let result = run_post_script(
            r#"
             test("Status is 200", status_code() == 200);
             test("Is JSON", get_header("Content-Type") == "application/json");
             "#,
            200,
            "{}",
            &headers,
            100,
        );

        assert_eq!(result.tests.len(), 2);
        assert_eq!(result.tests[0], ("Status is 200".to_string(), true));
        assert_eq!(result.tests[1], ("Is JSON".to_string(), true));
    }
}
