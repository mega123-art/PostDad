use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const GIST_DESC: &str = "PostDad Sync Backup";

#[derive(Debug, Serialize, Deserialize)]
struct GistFile {
    content: String,
}

#[derive(Debug, Serialize)]
struct GistCreateRequest {
    description: String,
    public: bool,
    files: HashMap<String, GistFile>,
}

#[derive(Debug, Serialize)]
struct GistUpdateRequest {
    description: String,
    files: HashMap<String, GistFile>,
}

#[derive(Debug, Deserialize)]
struct GistResponse {
    id: String,
    description: Option<String>,
}

pub async fn sync_to_gist(token: &str) -> Result<(), String> {
    let mut files = HashMap::new();

    // Read collections
    let col_path = Path::new("collections");
    if col_path.exists() {
        if let Ok(entries) = fs::read_dir(col_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("hcl") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let name = path.file_name().unwrap().to_string_lossy().into_owned();
                        files.insert(name, GistFile { content });
                    }
                }
            }
        }
    }

    // Read environments
    let env_path = Path::new("environments.hcl");
    if env_path.exists() {
        if let Ok(content) = fs::read_to_string(env_path) {
            files.insert("environments.hcl".to_string(), GistFile { content });
        }
    }

    if files.is_empty() {
        return Err("No collections or environments found to sync.".to_string());
    }

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Accept",
        header::HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        header::HeaderValue::from_static("2022-11-28"),
    );

    let mut auth_val =
        header::HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| e.to_string())?;
    auth_val.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_val);
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("PostDad"),
    );

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())?;

    // Find existing gist
    let res = client
        .get("https://api.github.com/gists")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to fetch gists: {}", res.status()));
    }

    let gists: Vec<GistResponse> = res.json().await.map_err(|e| e.to_string())?;

    let mut existing_gist_id = None;
    for gist in gists {
        if let Some(desc) = gist.description {
            if desc == GIST_DESC {
                existing_gist_id = Some(gist.id);
                break;
            }
        }
    }

    if let Some(id) = existing_gist_id {
        // Update gist
        let req = GistUpdateRequest {
            description: GIST_DESC.to_string(),
            files,
        };

        let res = client
            .patch(&format!("https://api.github.com/gists/{}", id))
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            println!("Successfully updated Gist backup ({})!", id);
            Ok(())
        } else {
            Err(format!("Failed to update gist: {}", res.status()))
        }
    } else {
        // Create new
        let req = GistCreateRequest {
            description: GIST_DESC.to_string(),
            public: false,
            files,
        };

        let res = client
            .post("https://api.github.com/gists")
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            let created: GistResponse = res.json().await.map_err(|e| e.to_string())?;
            println!("Successfully created new Gist backup ({})!", created.id);
            Ok(())
        } else {
            Err(format!("Failed to create gist: {}", res.status()))
        }
    }
}
