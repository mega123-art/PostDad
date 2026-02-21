use crate::features::cli::load_collection;
use serde_json::json;
use std::fs;

pub async fn export_to_postman(collection_path: &str, output_path: &str) -> Result<(), String> {
    let collection = load_collection(collection_path).await?;

    let mut items = Vec::new();

    for (name, config) in collection.requests {
        let url_obj = json!({
            "raw": config.url
        });

        let mut headers = Vec::new();
        if let Some(h) = config.headers {
            for (k, v) in h {
                headers.push(json!({
                    "key": k,
                    "value": v
                }));
            }
        }

        let body_obj = if let Some(body_text) = config.body {
            json!({
                "mode": "raw",
                "raw": body_text
            })
        } else if let Some(form_data) = config.form_data {
            let fd = form_data
                .into_iter()
                .map(|(k, v, _)| {
                    json!({
                        "key": k,
                        "value": v,
                        "type": "text"
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "mode": "formdata",
                "formdata": fd
            })
        } else {
            json!({})
        };

        items.push(json!({
            "name": name,
            "request": {
                "method": config.method,
                "header": headers,
                "url": url_obj,
                "body": body_obj
            }
        }));
    }

    let postman_json = json!({
        "info": {
            "name": collection.name,
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": items
    });

    let content = serde_json::to_string_pretty(&postman_json).map_err(|e| e.to_string())?;
    fs::write(output_path, content).map_err(|e| e.to_string())?;

    println!("Exported Postman collection to {}", output_path);
    Ok(())
}

pub async fn export_to_insomnia(collection_path: &str, output_path: &str) -> Result<(), String> {
    let collection = load_collection(collection_path).await?;

    let mut resources = Vec::new();

    let workspace_id = "wrk_1";
    resources.push(json!({
        "_type": "workspace",
        "_id": workspace_id,
        "name": collection.name
    }));

    let mut req_idx = 1;
    for (name, config) in collection.requests {
        let req_id = format!("req_{}", req_idx);
        req_idx += 1;

        let mut headers = Vec::new();
        if let Some(h) = config.headers {
            for (k, v) in h {
                headers.push(json!({
                    "name": k,
                    "value": v
                }));
            }
        }

        let body_obj = if let Some(body_text) = config.body {
            json!({
                "mimeType": "application/json",
                "text": body_text
            })
        } else if let Some(form_data) = config.form_data {
            let fd = form_data
                .into_iter()
                .map(|(k, v, _)| {
                    json!({
                        "name": k,
                        "value": v
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "mimeType": "multipart/form-data",
                "params": fd
            })
        } else {
            json!({})
        };

        resources.push(json!({
            "_type": "request",
            "_id": req_id,
            "parentId": workspace_id,
            "name": name,
            "url": config.url,
            "method": config.method,
            "body": body_obj,
            "headers": headers
        }));
    }

    let insomnia_json = json!({
        "_type": "export",
        "__export_format": 4,
        "__export_date": "2024-01-01T00:00:00.000Z",
        "__export_source": "postdad",
        "resources": resources
    });

    let content = serde_json::to_string_pretty(&insomnia_json).map_err(|e| e.to_string())?;
    fs::write(output_path, content).map_err(|e| e.to_string())?;

    println!("Exported Insomnia collection to {}", output_path);
    Ok(())
}
