use hcl::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
}

impl Environment {
    pub fn load_from_file(path: &str) -> std::io::Result<Vec<Environment>> {
        let path = Path::new(path);
        let mut envs = Vec::new();

        if !path.exists() {
            let default_hcl = r#"
env "dev" {
  base_url = "https://jsonplaceholder.typicode.com"
  token = "dev_token_123"
}

env "prod" {
  base_url = "https://api.github.com"
  token = "prod_secret_abc"
}
"#;
            fs::write(path, default_hcl)?;
        }

        let content = fs::read_to_string(path)?;
        let body: Body = hcl::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        for block in body.blocks() {
            if block.identifier() == "env" {
                if let Some(label) = block.labels().first() {
                    let variables: HashMap<String, String> =
                        hcl::from_body(block.body().clone())
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

                    envs.push(Environment {
                        name: label.as_str().to_string(),
                        variables,
                    });
                }
            }
        }

        envs.insert(
            0,
            Environment {
                name: "None".to_string(),
                variables: HashMap::new(),
            },
        );

        Ok(envs)
    }
}
