use serde::{Deserialize, Serialize};
use hcl::Body;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfig {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub requests: HashMap<String, RequestConfig>,
}

impl Collection {
    pub fn load_from_dir(path: &str) -> std::io::Result<Vec<Collection>> {
        let mut collections = Vec::new();
        let path = Path::new(path);

        if !path.exists() {
            fs::create_dir_all(path)?;
            // Create a default example
            let default_hcl = r#"
request "GitHub Zen" {
  url = "https://api.github.com/zen"
  method = "GET"
  headers = {
    "User-Agent" = "Postdad"
  }
}

request "JSONPlaceholder" {
  url = "https://jsonplaceholder.typicode.com/todos/1"
  method = "GET"
}
"#;
            fs::write(path.join("default.hcl"), default_hcl)?;
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hcl") {
                let content = fs::read_to_string(&path)?;
                let body: Body = hcl::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                
                let mut requests = HashMap::new();
                
                // Parse the HCL body manually to extract "request" blocks
                for block in body.blocks() {
                    if block.identifier() == "request" {
                        if let Some(label) = block.labels().first() {
                            let config: RequestConfig = hcl::from_body(block.body().clone())
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                            requests.insert(label.as_str().to_string(), config);
                        }
                    }
                }

                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                collections.push(Collection { name, requests });
            }
        }
        
        // Sort collections by name for consistency
        collections.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(collections)
    }
}
