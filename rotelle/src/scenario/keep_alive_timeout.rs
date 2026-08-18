use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::AbortHandle;

pub struct KeepAliveTimeoutScenario {
    idle_secs: Mutex<u64>,
    closed_connections: Arc<AtomicU64>,
    timer_task: Mutex<Option<AbortHandle>>,
}

impl KeepAliveTimeoutScenario {
    pub fn new() -> Self {
        Self {
            idle_secs: Mutex::new(10),
            closed_connections: Arc::new(AtomicU64::new(0)),
            timer_task: Mutex::new(None),
        }
    }
}

impl Scenario for KeepAliveTimeoutScenario {
    fn name(&self) -> &'static str {
        "keep-alive-timeout"
    }

    fn description(&self) -> &'static str {
        "Serves valid responses but closes the keep-alive connection each request, \
         simulating a pod whose keepalive idle timeout is shorter than the ingress/LB — \
         the classic intermittent 502 on connection reuse."
    }

    fn activate(&self, params: &ActivationParams) {
        let idle_secs = params.get_u64("idle_secs").unwrap_or(10);
        *self.idle_secs.lock().unwrap() = idle_secs;

        let closed = Arc::clone(&self.closed_connections);
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(idle_secs)).await;
                let total = closed.load(Ordering::Relaxed);
                tracing::info!(
                    idle_secs,
                    total_closed = total,
                    "keep-alive-timeout: idle window elapsed; connections closed on reuse"
                );
            }
        })
        .abort_handle();

        *self.timer_task.lock().unwrap() = Some(handle);
    }

    fn deactivate(&self) {
        if let Some(handle) = self.timer_task.lock().unwrap().take() {
            handle.abort();
        }
        self.closed_connections.store(0, Ordering::Relaxed);
    }

    fn on_index_request(&self) -> IndexEffect {
        let n = self.closed_connections.fetch_add(1, Ordering::Relaxed) + 1;
        IndexEffect::RespondThenClose(format!(
            "<p>Served OK — but this keep-alive connection is being closed.</p>\
             <p>Connections closed so far: <strong>{n}</strong></p>"
        ))
    }

    fn status_extras(&self) -> serde_json::Value {
        serde_json::json!({
            "idle_secs": *self.idle_secs.lock().unwrap(),
            "closed_connections": self.closed_connections.load(Ordering::Relaxed),
        })
    }

    fn default_params(&self) -> ActivationParams {
        ActivationParams::from_json(serde_json::json!({
            "idle_secs": 10
        }))
    }
}
