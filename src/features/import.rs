use crate::domain::collection::{Collection, RequestConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct PostmanCollection {
    info: Info,
    item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Info {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Item {
    name: String,
    request: Option<Request>,
    item: Option<Vec<Item>>, // For nested folders
}

#[derive(Debug, Deserialize)]
struct Request {
    method: String,
    header: Option<Vec<Header>>,
    url: Option<Url>,
    body: Option<Body>,
    auth: Option<Auth>,
}

#[derive(Debug, Deserialize)]
struct Auth {
    #[serde(rename = "type")]
    auth_type: String,
    bearer: Option<Vec<KeyValue>>,
}

#[derive(Debug, Deserialize)]
struct Header {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct KeyValue {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Url {
    String(String),
    Object {
        raw: String,
        // 'query' is redundant as 'raw' contains the full URL
    },
}

#[derive(Debug, Deserialize)]
struct Body {
    mode: Option<String>,
    raw: Option<String>,
    urlencoded: Option<Vec<KeyValue>>,
    formdata: Option<Vec<KeyValue>>,
}

pub fn import_postman_collection(file_path: &str) -> std::io::Result<()> {
    let content = fs::read_to_string(file_path)?;
    let pm_collection: PostmanCollection = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut requests = HashMap::new();

    // Flatten items
    flatten_items(&pm_collection.item, &mut requests, "");

    let collection = Collection {
        name: pm_collection.info.name.clone(),
        requests,
    };

    let safe_name = collection.name.replace(" ", "_").to_lowercase();
    let file_name = format!("collections/{}.hcl", safe_name);

    if !std::path::Path::new("collections").exists() {
        fs::create_dir("collections")?;
    }

    let mut hcl_content = String::new();

    for (name, config) in &collection.requests {
        let body_hcl = hcl::to_string(&config).map_err(std::io::Error::other)?;

        let entry = format!("\nrequest \"{}\" {{\n{}\n}}\n", name, body_hcl);
        hcl_content.push_str(&entry);
    }

    fs::write(&file_name, hcl_content)?;

    println!(
        "Successfully imported '{}' to '{}'",
        collection.name, file_name
    );

    Ok(())
}

fn flatten_items(items: &[Item], requests: &mut HashMap<String, RequestConfig>, prefix: &str) {
    for item in items {
        if let Some(req) = &item.request {
            // It's a request
            let name = if prefix.is_empty() {
                item.name.clone()
            } else {
                format!("{}/{}", prefix, item.name)
            };

            let url_str = match &req.url {
                Some(Url::String(s)) => s.clone(),
                Some(Url::Object { raw }) => raw.clone(),
                None => String::new(),
            };

            let mut headers_map = HashMap::new();
            if let Some(headers) = &req.header {
                for h in headers {
                    headers_map.insert(h.key.clone(), h.value.clone());
                }
            }

            // Handle Auth
            if let Some(auth) = &req.auth {
                if auth.auth_type == "bearer" {
                    if let Some(bearer) = &auth.bearer {
                        // Find the token value
                        if let Some(token) = bearer.iter().find(|kv| kv.key == "token") {
                            headers_map.insert(
                                "Authorization".to_string(),
                                format!("Bearer {}", token.value),
                            );
                        }
                    }
                } else if auth.auth_type == "basic" {
                }
            }

            let headers_opt = if headers_map.is_empty() {
                None
            } else {
                Some(headers_map)
            };

            // Handle Body
            let (body_str, form_data, body_type) = if let Some(body) = &req.body {
                match body.mode.as_deref() {
                    Some("formdata") => {
                        if let Some(fd) = &body.formdata {
                            let data: Vec<(String, String, bool)> = fd
                                .iter()
                                .map(|kv| (kv.key.clone(), kv.value.clone(), true))
                                .collect();
                            (None, Some(data), Some("FormData".to_string()))
                        } else {
                            (None, None, None)
                        }
                    }
                    Some("urlencoded") => {
                        if let Some(ue) = &body.urlencoded {
                            let params: Vec<String> = ue
                                .iter()
                                .map(|kv| format!("{}={}", kv.key, kv.value))
                                .collect();
                            (Some(params.join("&")), None, Some("Raw".to_string()))
                        } else {
                            (None, None, None)
                        }
                    }
                    _ => (body.raw.clone(), None, Some("Raw".to_string())),
                }
            } else {
                (None, None, None)
            };

            let config = RequestConfig {
                url: url_str,
                method: req.method.clone(),
                body: body_str,
                headers: headers_opt,
                extract: None,
                body_type,
                form_data,
                graphql_query: None,
                graphql_variables: None,
                expected_status: None,
                timeout_ms: None,
                pre_request_script: None,
                post_request_script: None,
            };

            requests.insert(name, config);
        } else if let Some(sub_items) = &item.item {
            // It's a folder
            let new_prefix = if prefix.is_empty() {
                item.name.clone()
            } else {
                format!("{}/{}", prefix, item.name)
            };
            flatten_items(sub_items, requests, &new_prefix);
        }
    }
}

// ============================================================================
// OpenAPI v3 Import
// ============================================================================

#[derive(Debug, Deserialize)]
struct OpenApiSpec {
    openapi: Option<String>,
    info: OpenApiInfo,
    servers: Option<Vec<OpenApiServer>>,
    paths: HashMap<String, HashMap<String, OpenApiOperation>>,
}

#[derive(Debug, Deserialize)]
struct OpenApiInfo {
    title: String,
    #[serde(default)]
    version: String,
}

#[derive(Debug, Deserialize)]
struct OpenApiServer {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Fields retained for OpenAPI spec completeness
struct OpenApiOperation {
    summary: Option<String>,
    operation_id: Option<String>,
    parameters: Option<Vec<OpenApiParameter>>,
    request_body: Option<OpenApiRequestBody>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields retained for OpenAPI spec completeness
struct OpenApiParameter {
    name: String,
    #[serde(rename = "in")]
    location: String, // "query", "header", "path", "cookie"
    required: Option<bool>,
    schema: Option<OpenApiSchema>,
}

#[derive(Debug, Deserialize)]
struct OpenApiRequestBody {
    content: Option<HashMap<String, OpenApiMediaType>>,
}

#[derive(Debug, Deserialize)]
struct OpenApiMediaType {
    schema: Option<OpenApiSchema>,
    example: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenApiSchema {
    #[serde(rename = "type")]
    schema_type: Option<String>,
    properties: Option<HashMap<String, OpenApiSchema>>,
    items: Option<Box<OpenApiSchema>>,
    #[serde(rename = "enum")]
    enum_values: Option<Vec<serde_json::Value>>,
    example: Option<serde_json::Value>,
    default: Option<serde_json::Value>,
}

pub fn import_openapi(file_path: &str) -> std::io::Result<()> {
    let content = fs::read_to_string(file_path)?;
    let spec: OpenApiSpec = serde_json::from_str(&content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid OpenAPI JSON: {}", e),
        )
    })?;

    // Validate it's OpenAPI v3
    if let Some(ref version) = spec.openapi
        && !version.starts_with("3.")
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Unsupported OpenAPI version: {}. Only v3.x is supported.",
                version
            ),
        ));
    }

    // Get base URL from servers
    let base_url = spec
        .servers
        .as_ref()
        .and_then(|s| s.first())
        .map(|s| s.url.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "https://api.example.com".to_string());

    let mut requests = HashMap::new();

    // Process each path and operation
    for (path, methods) in &spec.paths {
        for (method, operation) in methods {
            // Skip non-HTTP methods (like "parameters" at path level)
            let http_method = method.to_uppercase();
            if !["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
                .contains(&http_method.as_str())
            {
                continue;
            }

            // Generate request name
            let name = operation
                .operation_id
                .clone()
                .or_else(|| operation.summary.clone())
                .unwrap_or_else(|| format!("{} {}", http_method, path));

            // Build full URL with path
            let full_url = format!("{}{}", base_url, path);

            // Extract headers from parameters
            let mut headers_map = HashMap::new();
            let mut query_params = Vec::new();

            if let Some(params) = &operation.parameters {
                for param in params {
                    match param.location.as_str() {
                        "header" => {
                            let value = get_example_value(&param.schema);
                            headers_map.insert(param.name.clone(), value);
                        }
                        "query" => {
                            let value = get_example_value(&param.schema);
                            query_params.push((param.name.clone(), value));
                        }
                        _ => {} // path params are in URL, cookies ignored for now
                    }
                }
            }

            // Build URL with query params
            let url_with_params = if query_params.is_empty() {
                full_url
            } else {
                let qs: Vec<String> = query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                format!("{}?{}", full_url, qs.join("&"))
            };

            // Extract request body
            let (body_str, body_type) = if let Some(req_body) = &operation.request_body {
                if let Some(content) = &req_body.content {
                    if let Some(json_media) = content.get("application/json") {
                        // Add Content-Type header
                        headers_map
                            .insert("Content-Type".to_string(), "application/json".to_string());

                        // Get example or generate from schema
                        let body = if let Some(example) = &json_media.example {
                            serde_json::to_string_pretty(example).unwrap_or_default()
                        } else if let Some(schema) = &json_media.schema {
                            generate_example_from_schema(schema)
                        } else {
                            "{}".to_string()
                        };
                        (Some(body), Some("Raw".to_string()))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            let headers_opt = if headers_map.is_empty() {
                None
            } else {
                Some(headers_map)
            };

            let config = RequestConfig {
                url: url_with_params,
                method: http_method,
                body: body_str,
                headers: headers_opt,
                extract: None,
                body_type,
                form_data: None,
                graphql_query: None,
                graphql_variables: None,
                expected_status: None,
                timeout_ms: None,
                pre_request_script: None,
                post_request_script: None,
            };

            requests.insert(name, config);
        }
    }

    if requests.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No API operations found in the OpenAPI spec",
        ));
    }

    let collection = Collection {
        name: spec.info.title.clone(),
        requests,
    };

    let safe_name = collection.name.replace(" ", "_").to_lowercase();
    let file_name = format!("collections/{}.hcl", safe_name);

    if !std::path::Path::new("collections").exists() {
        fs::create_dir("collections")?;
    }

    let mut hcl_content = String::new();

    for (name, config) in &collection.requests {
        let body_hcl = hcl::to_string(&config).map_err(std::io::Error::other)?;

        let entry = format!("\nrequest \"{}\" {{\n{}\n}}\n", name, body_hcl);
        hcl_content.push_str(&entry);
    }

    fs::write(&file_name, hcl_content)?;

    println!(
        "Successfully imported OpenAPI spec '{}' v{} to '{}'",
        spec.info.title, spec.info.version, file_name
    );
    println!("  → {} requests created", collection.requests.len());

    Ok(())
}

/// Get example value from schema, or placeholder
fn get_example_value(schema: &Option<OpenApiSchema>) -> String {
    if let Some(s) = schema {
        if let Some(example) = &s.example {
            return example.to_string().trim_matches('"').to_string();
        }
        if let Some(default) = &s.default {
            return default.to_string().trim_matches('"').to_string();
        }
        if let Some(enum_vals) = &s.enum_values
            && let Some(first) = enum_vals.first()
        {
            return first.to_string().trim_matches('"').to_string();
        }
        // Return type-based placeholder
        match s.schema_type.as_deref() {
            Some("string") => "example".to_string(),
            Some("integer") | Some("number") => "0".to_string(),
            Some("boolean") => "true".to_string(),
            _ => "value".to_string(),
        }
    } else {
        "value".to_string()
    }
}

/// Generate example JSON from schema
fn generate_example_from_schema(schema: &OpenApiSchema) -> String {
    let value = schema_to_value(schema);
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
}

fn schema_to_value(schema: &OpenApiSchema) -> serde_json::Value {
    // Return example if available
    if let Some(example) = &schema.example {
        return example.clone();
    }
    if let Some(default) = &schema.default {
        return default.clone();
    }
    if let Some(enum_vals) = &schema.enum_values
        && let Some(first) = enum_vals.first()
    {
        return first.clone();
    }

    match schema.schema_type.as_deref() {
        Some("object") => {
            if let Some(props) = &schema.properties {
                let mut obj = serde_json::Map::new();
                for (key, prop_schema) in props {
                    obj.insert(key.clone(), schema_to_value(prop_schema));
                }
                serde_json::Value::Object(obj)
            } else {
                serde_json::json!({})
            }
        }
        Some("array") => {
            if let Some(items) = &schema.items {
                serde_json::Value::Array(vec![schema_to_value(items)])
            } else {
                serde_json::json!([])
            }
        }
        Some("string") => serde_json::Value::String("string".to_string()),
        Some("integer") => serde_json::json!(0),
        Some("number") => serde_json::json!(0.0),
        Some("boolean") => serde_json::json!(false),
        _ => serde_json::Value::Null,
    }
}

/// Auto-detect file format and import accordingly
pub fn import_auto(file_path: &str) -> std::io::Result<()> {
    let content = fs::read_to_string(file_path)?;

    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        // Check for OpenAPI v3 signature
        if json.get("openapi").is_some() {
            println!("Detected OpenAPI v3 format");
            return import_openapi(file_path);
        }
        // Check for Postman signature
        if json.get("info").is_some() && json.get("item").is_some() {
            println!("Detected Postman Collection format");
            return import_postman_collection(file_path);
        }
    }

    // Default to Postman for backwards compatibility
    println!("Format not detected, attempting Postman import...");
    import_postman_collection(file_path)
}
