// CLI mode for running requests without the TUI
use crate::domain::collection::Collection;
use crate::domain::environment::Environment;
use crate::features::runner::{self, RunResult, RunnerEvent};
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::mpsc;

/// ANSI color codes for terminal output
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const GREEN: &str = "\x1b[32m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
    pub const MAGENTA: &str = "\x1b[35m";
}

/// CLI arguments for run command
pub struct RunArgs {
    pub collection_path: String,
    pub env_path: Option<String>,
    pub verbose: bool,
    pub json_output: bool,
}

/// Parse CLI arguments and return the action to take
pub fn parse_args() -> Option<CliAction> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        return None; // No args, launch TUI
    }
    
    match args[1].as_str() {
        "--import" => {
            if args.len() >= 3 {
                Some(CliAction::Import(args[2].clone()))
            } else {
                eprintln!("Usage: PostDad --import <postman_collection.json>");
                std::process::exit(1);
            }
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: PostDad run <collection.hcl> [-e env.hcl] [-v] [--json]");
                std::process::exit(1);
            }
            
            let collection_path = args[2].clone();
            let mut env_path = None;
            let mut verbose = false;
            let mut json_output = false;
            
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "-e" | "--env" => {
                        if i + 1 < args.len() {
                            env_path = Some(args[i + 1].clone());
                            i += 1;
                        }
                    }
                    "-v" | "--verbose" => verbose = true,
                    "--json" => json_output = true,
                    _ => {}
                }
                i += 1;
            }
            
            Some(CliAction::Run(RunArgs {
                collection_path,
                env_path,
                verbose,
                json_output,
            }))
        }
        "--help" | "-h" => {
            print_help();
            std::process::exit(0);
        }
        "--version" | "-V" => {
            println!("PostDad {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        _ => None, // Unknown arg, launch TUI
    }
}

pub enum CliAction {
    Import(String),
    Run(RunArgs),
}

fn print_help() {
    println!(
        r#"{}PostDad{} - A fast API client for your terminal

{}USAGE:{}
    PostDad                              Launch the TUI
    PostDad run <collection.hcl>         Run a collection
    PostDad --import <file.json>         Import a Postman collection

{}OPTIONS:{}
    -e, --env <file.hcl>    Environment file to use
    -v, --verbose           Show request/response details
    --json                  Output results as JSON
    -h, --help              Show this help
    -V, --version           Show version

{}EXAMPLES:{}
    PostDad run api_tests.hcl
    PostDad run api_tests.hcl -e production.hcl
    PostDad run api_tests.hcl --json > results.json
"#,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
    );
}

/// Run a collection in CLI mode
pub async fn run_collection_cli(args: RunArgs) -> i32 {
    // Load collection
    let collection = match load_collection(&args.collection_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}Error:{} Failed to load collection: {}", colors::RED, colors::RESET, e);
            return 1;
        }
    };
    
    // Load environment if specified
    let env_vars: HashMap<String, String> = if let Some(env_path) = &args.env_path {
        match load_environment(env_path) {
            Ok(vars) => vars,
            Err(e) => {
                eprintln!("{}Error:{} Failed to load environment: {}", colors::RED, colors::RESET, e);
                return 1;
            }
        }
    } else {
        HashMap::new()
    };
    
    let total_requests = collection.requests.len();
    
    if !args.json_output {
        println!();
        println!("{}▶ Running:{} {} ({} requests)", 
            colors::CYAN, colors::RESET, 
            collection.name, total_requests);
        println!("{}{}{}", colors::DIM, "─".repeat(50), colors::RESET);
    }
    
    // Create channel for runner events
    let (tx, mut rx) = mpsc::channel::<RunnerEvent>(32);
    
    // Spawn the runner
    let collection_clone = collection.clone();
    let env_vars_clone = env_vars.clone();
    tokio::spawn(async move {
        runner::run_collection(&collection_clone, &env_vars_clone, tx).await;
    });
    
    let mut results: Vec<RunResult> = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    
    // Process events
    while let Some(event) = rx.recv().await {
        match event {
            RunnerEvent::RequestCompleted(result) => {
                if !args.json_output {
                    print_result(&result, args.verbose);
                }
                
                if result.passed {
                    passed += 1;
                } else {
                    failed += 1;
                }
                results.push(result);
            }
            RunnerEvent::Finished(_) => break,
            RunnerEvent::Error(e) => {
                if args.json_output {
                    println!(r#"{{"error": "{}"}}"#, e);
                } else {
                    eprintln!("{}Error:{} {}", colors::RED, colors::RESET, e);
                }
                return 1;
            }
            _ => {}
        }
    }
    
    // Output results
    if args.json_output {
        print_json_results(&collection.name, &results, passed, failed);
    } else {
        println!("{}{}{}", colors::DIM, "─".repeat(50), colors::RESET);
        print_summary(passed, failed, total_requests);
    }
    
    // Exit code: 0 if all passed, 1 if any failed
    if failed > 0 { 1 } else { 0 }
}

fn load_collection(path: &str) -> Result<Collection, String> {
    let path = Path::new(path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let body: hcl::Body = hcl::from_str(&content)
        .map_err(|e| format!("Failed to parse HCL: {}", e))?;
    
    let mut requests = HashMap::new();
    
    for block in body.blocks() {
        if block.identifier() == "request"
            && let Some(label) = block.labels().first() {
                let config: crate::domain::collection::RequestConfig = hcl::from_body(block.body().clone())
                    .map_err(|e| format!("Failed to parse request '{}': {}", label.as_str(), e))?;
                requests.insert(label.as_str().to_string(), config);
            }
    }
    
    if requests.is_empty() {
        return Err("No requests found in collection".to_string());
    }
    
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("collection")
        .to_string();
    
    Ok(Collection { name, requests })
}

fn load_environment(path: &str) -> Result<HashMap<String, String>, String> {
    let envs = Environment::load_from_file(path)
        .map_err(|e| format!("Failed to load environment: {}", e))?;
    
    // Use the first non-"None" environment, or merge all
    let mut vars = HashMap::new();
    for env in envs {
        if env.name != "None" {
            vars.extend(env.variables);
            break; // Use first environment found
        }
    }
    
    Ok(vars)
}

fn print_result(result: &RunResult, verbose: bool) {
    let status_icon = if result.passed {
        format!("{}✓{}", colors::GREEN, colors::RESET)
    } else {
        format!("{}✗{}", colors::RED, colors::RESET)
    };
    
    let status_code = result.status
        .map(|s| format!("{}", s))
        .unwrap_or_else(|| "ERR".to_string());
    
    let status_color = match result.status {
        Some(s) if (200..300).contains(&s) => colors::GREEN,
        Some(s) if s >= 400 => colors::RED,
        Some(_) => colors::YELLOW,
        None => colors::RED,
    };
    
    let latency = result.latency_ms
        .map(|ms| format!("{}ms", ms))
        .unwrap_or_else(|| "-".to_string());
    
    println!(
        "  {} {}{:>3}{} {}{:6}{} {} {}{}{}",
        status_icon,
        status_color, status_code, colors::RESET,
        colors::MAGENTA, result.method, colors::RESET,
        result.name,
        colors::DIM, latency, colors::RESET
    );
    
    // Print test results if any
    for (test_name, test_passed) in &result.tests {
        let test_icon = if *test_passed {
            format!("{}✓{}", colors::GREEN, colors::RESET)
        } else {
            format!("{}✗{}", colors::RED, colors::RESET)
        };
        println!("      {} {}", test_icon, test_name);
    }
    
    // Print error if any
    if let Some(ref error) = result.error {
        println!("      {}Error: {}{}", colors::RED, error, colors::RESET);
    }
    
    // Verbose: show URL
    if verbose {
        println!("      {}→ {}{}", colors::DIM, result.url, colors::RESET);
    }
}

fn print_summary(passed: usize, failed: usize, total: usize) {
    println!();
    
    if failed == 0 {
        println!(
            "{}✓ All {} requests passed{}",
            colors::GREEN, total, colors::RESET
        );
    } else {
        println!(
            "{}Summary:{} {}{} passed{}, {}{} failed{}",
            colors::BOLD, colors::RESET,
            colors::GREEN, passed, colors::RESET,
            colors::RED, failed, colors::RESET
        );
    }
    
    println!();
}

fn print_json_results(collection_name: &str, results: &[RunResult], passed: usize, failed: usize) {
    let results_json: Vec<serde_json::Value> = results.iter().map(|r| {
        serde_json::json!({
            "name": r.name,
            "method": r.method,
            "url": r.url,
            "status": r.status,
            "latency_ms": r.latency_ms,
            "expected_status": r.expected_status,
            "passed": r.passed,
            "error": r.error,
            "tests": r.tests.iter().map(|(name, passed)| {
                serde_json::json!({"name": name, "passed": passed})
            }).collect::<Vec<_>>()
        })
    }).collect();
    
    let output = serde_json::json!({
        "collection": collection_name,
        "total": results.len(),
        "passed": passed,
        "failed": failed,
        "results": results_json
    });
    
    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
}
