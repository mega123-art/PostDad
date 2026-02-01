use crate::app::{App, AuthType, BodyType};

/// Helper to create a fully populated App with a specific active tab configuration
fn create_test_app() -> App {
    let mut app = App::new();
    // Clear default tab or use it
    let tab = app.active_tab_mut();

    tab.url = "https://api.example.com/v1/resource".to_string();
    tab.method = "POST".to_string();

    // Add Headers
    tab.request_headers
        .insert("Content-Type".to_string(), "application/json".to_string());
    tab.request_headers
        .insert("X-Custom-Header".to_string(), "custom_value".to_string());

    // Add Body
    tab.body_type = BodyType::Raw;
    tab.request_body = "{\"key\": \"value\"}".to_string();

    // Add Auth
    tab.auth_type = AuthType::Bearer;
    tab.auth_token = "test_token_123".to_string();

    app
}

#[test]
fn test_generate_curl() {
    let app = create_test_app();
    let code = app.generate_curl_command();

    assert!(code.contains("curl -X POST \"https://api.example.com/v1/resource\""));
    assert!(code.contains("-H \"Authorization: Bearer test_token_123\""));
    assert!(code.contains("-H \"Content-Type: application/json\""));
    assert!(code.contains("-d '{\"key\": \"value\"}'"));
}

#[test]
fn test_generate_python() {
    let app = create_test_app();
    let code = app.generate_python_code();

    assert!(code.contains("import requests"));
    assert!(code.contains("url = \"https://api.example.com/v1/resource\""));
    assert!(code.contains("\"Authorization\": \"Bearer test_token_123\""));
    assert!(code.contains("payload = '''{\"key\": \"value\"}'''"));
    assert!(code.contains("requests.request(\"POST\", url"));
}

#[test]
fn test_generate_javascript() {
    let app = create_test_app();
    let code = app.generate_javascript_code();

    assert!(code.contains("const url = \"https://api.example.com/v1/resource\";"));
    assert!(code.contains("method: 'POST'"));
    assert!(code.contains("'Authorization': 'Bearer test_token_123'"));
    assert!(code.contains("JSON.stringify({\"key\": \"value\"})"));
}

#[test]
fn test_generate_go() {
    let app = create_test_app();
    let code = app.generate_go_code();

    assert!(code.contains("package main"));
    assert!(code.contains("net/http"));
    assert!(code.contains("url := \"https://api.example.com/v1/resource\""));
    assert!(code.contains("req.Header.Add(\"Authorization\", \"Bearer test_token_123\")"));
    assert!(code.contains("strings.NewReader(`{\"key\": \"value\"}`)"));
}

#[test]
fn test_generate_rust() {
    let app = create_test_app();
    let code = app.generate_rust_code();

    assert!(code.contains("reqwest::Client::new()"));
    assert!(code.contains("reqwest::Method::POST"));
    assert!(code.contains(".bearer_auth(\"test_token_123\")"));
    assert!(code.contains(".body(\"{\\\"key\\\": \\\"value\\\"}\")"));
}

#[test]
fn test_generate_ruby() {
    let app = create_test_app();
    let code = app.generate_ruby_code();

    assert!(code.contains("require 'net/http'"));
    assert!(code.contains("Net::HTTP::Post.new(url)"));
    assert!(code.contains("request[\"Authorization\"] = \"Bearer test_token_123\""));
    assert!(code.contains("request.body = \"{\\\"key\\\": \\\"value\\\"}\""));
}

#[test]
fn test_generate_php() {
    let app = create_test_app();
    let code = app.generate_php_code();

    assert!(code.contains("curl_init()"));
    assert!(code.contains("CURLOPT_CUSTOMREQUEST => 'POST'"));
    assert!(code.contains("'Authorization: Bearer test_token_123'"));
    assert!(code.contains("CURLOPT_POSTFIELDS => '{\"key\": \"value\"}'"));
}

#[test]
fn test_generate_csharp() {
    let app = create_test_app();
    let code = app.generate_csharp_code();

    assert!(code.contains("new HttpClient()"));
    assert!(code.contains("HttpMethod.Post"));
    assert!(code.contains("Headers.Add(\"Authorization\", \"Bearer test_token_123\")"));
    assert!(code.contains("new StringContent(\"{\\\"key\\\": \\\"value\\\"}\""));
}
