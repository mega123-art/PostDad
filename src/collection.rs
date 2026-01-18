use hcl::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfig {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub extract: Option<HashMap<String, String>>,
    pub body_type: Option<String>,
    pub form_data: Option<Vec<(String, String, bool)>>,
    pub graphql_query: Option<String>,
    pub graphql_variables: Option<String>,
    #[serde(default)]
    pub expected_status: Option<u16>,
    pub pre_request_script: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub requests: HashMap<String, RequestConfig>,
}

impl Collection {
    // ... (load_from_dir stays the same, it derives Deserialize)

    pub fn load_from_dir(dir: &str) -> std::io::Result<Vec<Collection>> {
        let path = Path::new(dir);
        if !path.exists() {
            fs::create_dir_all(path)?;
        }

        let default_file_path = path.join("default.hcl");
        if !default_file_path.exists() {
            let default_hcl = r#"
request "GitHub Zen" {
  method = "GET"
  url = "https://api.github.com/zen"
}

request "JSONPlaceholder" {
  method = "GET"
  url = "https://jsonplaceholder.typicode.com/posts/1"
}
"#;
            fs::write(default_file_path, default_hcl)?;
        }

        let mut collections = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hcl") {
                let content = fs::read_to_string(&path)?;

                let body: Body = hcl::from_str(&content)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

                let mut requests = HashMap::new();

                for block in body.blocks() {
                    if block.identifier() == "request" {
                        if let Some(label) = block.labels().first() {
                            let config: RequestConfig = hcl::from_body(block.body().clone())
                                .map_err(|e| {
                                    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                                })?;
                            requests.insert(label.as_str().to_string(), config);
                        }
                    }
                }

                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                collections.push(Collection { name, requests });
            }
        }

        collections.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(collections)
    }

    pub fn save_to_file(
        name: &str,
        method: &str,
        url: &str,
        body: &str,
        headers: &HashMap<String, String>,
        extract: &Vec<(String, String)>,
        form_data: &Vec<(String, String, bool)>,
        body_type: &str,
        graphql_query: &str,
        graphql_variables: &str,
        pre_request_script: &str,
    ) -> std::io::Result<()> {
        let path = Path::new("collections/saved.hcl");

        let extract_map = if extract.is_empty() {
            None
        } else {
            Some(extract.iter().cloned().collect::<HashMap<_, _>>())
        };

        let headers_map = if headers.is_empty() {
            None
        } else {
            Some(headers.clone())
        };

        let form_data_opt = if form_data.is_empty() {
            None
        } else {
            Some(form_data.clone())
        };

        let body_opt = if body.trim().is_empty() {
            None
        } else {
            Some(body.to_string())
        };
        let body_type_opt = if body_type == "Raw" {
            None
        } else {
            Some(body_type.to_string())
        };

        let graphql_query_opt = if graphql_query.trim().is_empty() {
            None
        } else {
            Some(graphql_query.to_string())
        };
        let graphql_variables_opt = if graphql_variables.trim().is_empty() {
            None
        } else {
            Some(graphql_variables.to_string())
        };

        let pre_request_script_opt = if pre_request_script.trim().is_empty() {
            None
        } else {
            Some(pre_request_script.to_string())
        };

        let config = RequestConfig {
            url: url.to_string(),
            method: method.to_string(),
            body: body_opt,
            headers: headers_map,
            extract: extract_map,
            body_type: body_type_opt,
            form_data: form_data_opt,
            graphql_query: graphql_query_opt,
            graphql_variables: graphql_variables_opt,
            expected_status: None,
            pre_request_script: pre_request_script_opt,
        };

        let body_hcl = hcl::to_string(&config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let entry = format!("\nrequest \"{}\" {{\n{}\n}}\n", name, body_hcl);

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        file.write_all(entry.as_bytes())?;
        Ok(())
    }
}
