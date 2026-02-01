use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::Filter;

#[derive(Clone, Debug)]
pub struct MockRoute {
    pub path: String,
    pub method: String,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

pub struct MockServerHandle {
    pub handle: tokio::task::JoinHandle<()>,
}

pub fn start_mock_server(port: u16, routes: Vec<MockRoute>) -> MockServerHandle {
    let routes_state = Arc::new(Mutex::new(routes));
    let state_filter = warp::any().map(move || routes_state.clone());

    let handler = warp::any()
        .and(warp::path::full())
        .and(warp::method())
        .and(state_filter)
        .map(|path: warp::path::FullPath, method: warp::http::Method, state: Arc<Mutex<Vec<MockRoute>>>| {
            let path_str = path.as_str();
            let method_str = method.as_str();

            let routes = state.lock().unwrap();
            
            if let Some(route) = routes.iter().find(|r| {
                 r.path == path_str && r.method == method_str
            }) {
                let mut resp = warp::http::Response::builder()
                    .status(route.status);
                
                for (k, v) in &route.headers {
                    resp = resp.header(k, v);
                }

                resp.body(route.body.clone())
                    .unwrap_or_else(|_| warp::http::Response::new("Internal Server Error".to_string()))
            } else {
                warp::http::Response::builder()
                    .status(404)
                    .body(format!("Mock Not Found: {} {}", method_str, path_str))
                    .unwrap()
            }
        });

    let handle = tokio::spawn(warp::serve(handler).run(([127, 0, 0, 1], port)));

    MockServerHandle {
        handle,
    }
}
