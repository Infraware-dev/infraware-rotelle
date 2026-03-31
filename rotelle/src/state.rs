use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct State {
    pub failure_case: Arc<Mutex<String>>,
    pub state_file: String,
}

pub fn load_state(path: &str) -> String {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v["failure_case"].as_str().map(String::from))
        .unwrap_or_else(|| "none, idle".to_string())
}

pub fn persist_state(path: &str, case: &str) {
    let _ = std::fs::write(path, serde_json::json!({ "failure_case": case }).to_string());
}
