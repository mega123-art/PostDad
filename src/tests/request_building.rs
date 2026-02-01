use crate::app::{App, BodyType, AuthType};

#[test]
fn test_curl_import_basic() {
    let mut app = App::new();
    let curl = "curl https://api.example.com/data";
    
    // Check if returns Ok
    assert!(app.import_from_curl(curl).is_ok());
    
    let tab = app.active_tab();
    assert_eq!(tab.url, "https://api.example.com/data");
    assert_eq!(tab.method, "GET");
}

#[test]
fn test_curl_import_complex() {
    let mut app = App::new();
    // Test with newlines, continuations, headers, data
    let curl = r#"curl -X POST https://api.example.com/users \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        -d '{"name": "Alice"}' \
        -u "admin:secret""#;
        
    assert!(app.import_from_curl(curl).is_ok());
    
    let tab = app.active_tab();
    assert_eq!(tab.url, "https://api.example.com/users");
    assert_eq!(tab.method, "POST");
    
    // Check headers - keys might be case sensitive depending on implementation or normalized
    // In network.rs/handler.rs keys are usually kept as is. 
    // tokenize_curl implementation logic:
    // headers.push((key, value));
    
    let ct = tab.request_headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-type"));
    assert!(ct.is_some());
    assert_eq!(ct.unwrap().1, "application/json");
    
    // Check body
    assert_eq!(tab.body_type, BodyType::Raw);
    assert_eq!(tab.request_body, "{\"name\": \"Alice\"}");
    
    // Check Auth
    assert_eq!(tab.auth_type, AuthType::Basic);
    assert_eq!(tab.basic_auth_user, "admin");
    assert_eq!(tab.basic_auth_pass, "secret");
}

#[test]
fn test_curl_import_form_data() {
    let mut app = App::new();
    let curl = r#"curl -F "file=@/path/to/img.png" -F "userid=123" https://upload.com"#;
    
    assert!(app.import_from_curl(curl).is_ok());
    
    let tab = app.active_tab();
    assert_eq!(tab.method, "POST"); // Implicit POST with -F
    assert_eq!(tab.body_type, BodyType::FormData);
    
    assert_eq!(tab.form_data.len(), 2);
    // Find file
    let file_part = tab.form_data.iter().find(|(k, _, _)| k == "file").unwrap();
    assert_eq!(file_part.1, "/path/to/img.png");
    assert_eq!(file_part.2, true); // is_file
    
    // Find text
    let text_part = tab.form_data.iter().find(|(k, _, _)| k == "userid").unwrap();
    assert_eq!(text_part.1, "123");
    assert_eq!(text_part.2, false); // !is_file
}

#[test]
fn test_curl_import_failures() {
    let mut app = App::new();
    assert!(app.import_from_curl("wget https://example.com").is_err());
    assert!(app.import_from_curl("curl").is_err()); // no URL
}

#[test]
fn test_curl_quotes_handling() {
    let mut app = App::new();
    let curl = "curl -H 'X-Header: value with spaces' 'https://example.com/path?query=1'";
    
    assert!(app.import_from_curl(curl).is_ok());
    let tab = app.active_tab();
    
    assert_eq!(tab.url, "https://example.com/path?query=1");
    // Headers are stored in HashMap.
    // Iterating to find it or get directly.
    // Note: Implementation puts headers into tab.request_headers (HashMap<String, String>)
    
    let val = tab.request_headers.get("X-Header");
    assert!(val.is_some());
    assert_eq!(val.unwrap(), "value with spaces");
}
