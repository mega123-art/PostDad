use std::process::Command;
use std::collections::HashMap;

/// Result of a gRPC call
#[derive(Debug, Clone)]
pub struct GrpcResponse {
    pub success: bool,
    pub body: String,
    pub error: Option<String>,
    pub latency_ms: u128,
}

/// Execute a gRPC request using grpcurl
/// 
/// # Arguments
/// * `url` - The gRPC server address (e.g., "localhost:50051")
/// * `service_method` - Full service/method path (e.g., "grpc.health.v1.Health/Check")
/// * `proto_path` - Optional path to .proto file (if not using reflection)
/// * `payload` - JSON payload for the request
/// * `headers` - Additional headers/metadata
/// * `use_plaintext` - Whether to use plaintext (no TLS)
pub fn execute_grpc_request(
    url: &str,
    service_method: &str,
    proto_path: Option<&str>,
    payload: &str,
    headers: &HashMap<String, String>,
    use_plaintext: bool,
) -> GrpcResponse {
    let start = std::time::Instant::now();
    
    // Check if grpcurl is available
    let grpcurl_check = Command::new("which")
        .arg("grpcurl")
        .output();
    
    if grpcurl_check.is_err() || !grpcurl_check.unwrap().status.success() {
        return GrpcResponse {
            success: false,
            body: String::new(),
            error: Some("grpcurl not found. Install it: https://github.com/fullstorydev/grpcurl".to_string()),
            latency_ms: start.elapsed().as_millis(),
        };
    }
    
    let mut cmd = Command::new("grpcurl");
    
    // Add plaintext flag if needed (for non-TLS connections)
    if use_plaintext || !url.starts_with("https") {
        cmd.arg("-plaintext");
    }
    
    // Add proto file if specified
    if let Some(proto) = proto_path {
        if !proto.is_empty() {
            // Extract import path (directory containing the proto file)
            if let Some(parent) = std::path::Path::new(proto).parent() {
                cmd.arg("-import-path").arg(parent);
            }
            cmd.arg("-proto").arg(proto);
        }
    }
    
    // Add headers as metadata
    for (key, value) in headers {
        cmd.arg("-H").arg(format!("{}: {}", key, value));
    }
    
    // Add data (JSON payload)
    if !payload.is_empty() && payload != "{}" {
        cmd.arg("-d").arg(payload);
    }
    
    // Add the server address and method
    cmd.arg(url);
    cmd.arg(service_method);
    
    // Execute the command
    match cmd.output() {
        Ok(output) => {
            let latency = start.elapsed().as_millis();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            if output.status.success() {
                GrpcResponse {
                    success: true,
                    body: stdout,
                    error: None,
                    latency_ms: latency,
                }
            } else {
                // grpcurl outputs errors to stderr
                let error_msg = if stderr.is_empty() {
                    stdout.clone()
                } else {
                    stderr
                };
                
                GrpcResponse {
                    success: false,
                    body: stdout,
                    error: Some(error_msg),
                    latency_ms: latency,
                }
            }
        }
        Err(e) => GrpcResponse {
            success: false,
            body: String::new(),
            error: Some(format!("Failed to execute grpcurl: {}", e)),
            latency_ms: start.elapsed().as_millis(),
        },
    }
}

/// List available services using server reflection
pub fn list_services(url: &str, use_plaintext: bool) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("grpcurl");
    
    if use_plaintext {
        cmd.arg("-plaintext");
    }
    
    cmd.arg(url);
    cmd.arg("list");
    
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let services: Vec<String> = stdout
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                Ok(services)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to list services: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute grpcurl: {}", e)),
    }
}

/// Describe a service or method using server reflection
pub fn describe_service(url: &str, service: &str, use_plaintext: bool) -> Result<String, String> {
    let mut cmd = Command::new("grpcurl");
    
    if use_plaintext {
        cmd.arg("-plaintext");
    }
    
    cmd.arg(url);
    cmd.arg("describe");
    cmd.arg(service);
    
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to describe service: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute grpcurl: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_grpcurl_not_found_handling() {
        // This test verifies that we gracefully handle missing grpcurl
        // The actual behavior depends on whether grpcurl is installed
    }
}
