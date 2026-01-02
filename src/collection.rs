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
    pub fn load_from_dir(dir: &str) -> std::io::Result<Vec<Collection>> {
        let path = Path::new(dir);
        if !path.exists() {
            fs::create_dir_all(path)?;
            
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
            fs::write(path.join("default.hcl"), default_hcl)?;
        }

        let mut collections = Vec::new();
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hcl") {
                 let content = fs::read_to_string(&path)?;
                 
                 
                 let body: Body = hcl::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                 
                 let mut requests = HashMap::new();
                 
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
        
        
        collections.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(collections)
    }
    
    
    pub fn save_to_file(name: &str, method: &str, url: &str, body: &str, _headers: &HashMap<String, String>) -> std::io::Result<()> {
         
         let path = Path::new("collections/saved.hcl");
         
         let body_attr = if body.trim().is_empty() { 
             "".to_string() 
         } else {
             
             format!("\n  body = {:#?}", body) 
         };

         let entry = format!(
r#"
request "{}" {{
  method = "{}"
  url = "{}"{}
}}
"#, name, method, url, body_attr);

         use std::io::Write;
         let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
            
         file.write_all(entry.as_bytes())?;
         Ok(())
    }
}
