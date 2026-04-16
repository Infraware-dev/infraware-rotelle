use std::sync::{Arc, Mutex};
use tokio::task::AbortHandle;
use tracing::info;

#[derive(Clone)]
pub struct State {
    pub failure_case: Arc<Mutex<String>>,
    pub state_file: String,
    /// intermittent-01: counts GET / requests; crashes on every 5th
    pub access_count: Arc<Mutex<u32>>,
    /// intermittent-02: accumulates allocated chunks to prevent deallocation
    pub memory_sink: Arc<Mutex<Vec<Vec<u8>>>>,
    /// intermittent-02: handle to abort the background allocation task
    pub leak_task: Arc<Mutex<Option<AbortHandle>>>,
}

impl State {
    /// Stop any active behavioral mode side-effects (leak task, counters).
    /// Call before switching to any new failure case.
    pub fn reset_behavioral(&self) {
        if let Some(handle) = self.leak_task.lock().unwrap().take() {
            handle.abort();
        }
        self.memory_sink.lock().unwrap().clear();
        *self.access_count.lock().unwrap() = 0;
    }
}

pub fn load_state(path: &str) -> String {
    let loaded = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v["failure_case"].as_str().map(String::from));

    if let Some(ref case) = loaded {
        info!(path, failure_case = case, "failure case loaded from data");
    }

    loaded.unwrap_or_else(|| "none, idle".to_string())
}

pub fn persist_state(path: &str, case: &str) {
    let _ = std::fs::write(
        path,
        serde_json::json!({ "failure_case": case }).to_string(),
    );
}
