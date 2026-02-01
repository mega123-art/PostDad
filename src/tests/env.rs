use crate::app::App;

fn create_app_with_env(vars: Vec<(&str, &str)>) -> App {
    let mut app = App::new();
    // Use the active environment (index 0 usually "None", let's use index 0 and populate it for test)
    // App::new() might load envs, but let's just make sure we have one.
    if app.environments.is_empty() {
        app.environments.push(crate::environment::Environment {
            name: "Test".to_string(),
            variables: std::collections::HashMap::new(),
        });
    }
    
    // Use the currently selected environment
    let idx = app.selected_env_index;
    if let Some(env) = app.environments.get_mut(idx) {
        for (k, v) in vars {
            env.variables.insert(k.to_string(), v.to_string());
        }
    }
    app
}

#[test]
fn test_basic_substitution() {
    let mut app = create_app_with_env(vec![
        ("baseUrl", "https://api.example.com"),
        ("version", "v1")
    ]);
    
    app.active_tab_mut().url = "{{baseUrl}}/{{version}}/users".to_string();
    
    let processed = app.process_url();
    assert_eq!(processed, "https://api.example.com/v1/users");
}

#[test]
fn test_no_substitution_needed() {
    let mut app = create_app_with_env(vec![
        ("baseUrl", "https://api.example.com")
    ]);
    
    app.active_tab_mut().url = "https://google.com/search".to_string();
    
    let processed = app.process_url();
    assert_eq!(processed, "https://google.com/search");
}

#[test]
fn test_missing_variable() {
    let mut app = create_app_with_env(vec![
        ("baseUrl", "https://api.example.com")
    ]);
    
    app.active_tab_mut().url = "{{baseUrl}}/{{missingVar}}/users".to_string();
    
    let processed = app.process_url();
    assert_eq!(processed, "https://api.example.com/{{missingVar}}/users");
}

#[test]
fn test_multiple_occurrences() {
    let mut app = create_app_with_env(vec![
        ("id", "123")
    ]);
    
    app.active_tab_mut().url = "https://api.com/users/{{id}}/posts/{{id}}".to_string();
    
    let processed = app.process_url();
    assert_eq!(processed, "https://api.com/users/123/posts/123");
}

#[test]
fn test_partial_match_should_not_substitute() {
    let mut app = create_app_with_env(vec![
        ("base", "BASIC"),
        ("baseUrl", "FULL")
    ]);
    
    app.active_tab_mut().url = "{{base}} vs {{baseUrl}}".to_string();
    
    let processed = app.process_url();
    // Since HashMap iteration order is arbitrary, we need to ensure this works regardless of order.
    // However, string replacement of "{{base}}" will not match "{{baseUrl}}" because of the closing braces.
    // "{{base}}" matches exactly "{{base}}".
    assert_eq!(processed, "BASIC vs FULL");
}
