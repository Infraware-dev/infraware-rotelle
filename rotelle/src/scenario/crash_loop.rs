use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::Mutex;

const CRASH_EVERY: u32 = 5;

/// Simulates intermittent pod crashes: exits on every N-th GET /.
///
/// Kubernetes detects the non-zero exit code and restarts the pod.
pub struct CrashLoopScenario {
    access_count: Mutex<u32>,
}

impl CrashLoopScenario {
    pub fn new() -> Self {
        Self {
            access_count: Mutex::new(0),
        }
    }
}

impl Scenario for CrashLoopScenario {
    fn name(&self) -> &'static str {
        "crash-loop"
    }

    fn description(&self) -> &'static str {
        "Process exits on every 5th GET / — Kubernetes restarts the pod."
    }

    fn activate(&self, _params: &ActivationParams) {
        *self.access_count.lock().unwrap() = 0;
    }

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        let mut count = self.access_count.lock().unwrap();
        *count += 1;
        let n = *count;
        tracing::info!(count = n, "crash-loop: index access");
        if n.is_multiple_of(CRASH_EVERY) {
            tracing::warn!(count = n, "crash-loop: simulating crash");
            IndexEffect::Exit(1)
        } else {
            let next_crash = (n / CRASH_EVERY + 1) * CRASH_EVERY;
            IndexEffect::Respond(format!(
                "<p>Crash every <strong>N={CRASH_EVERY}</strong> accesses. \
                 Current count: <strong>{n}</strong>. \
                 Next crash at: <strong>{next_crash}</strong></p>"
            ))
        }
    }

    fn status_extras(&self) -> serde_json::Value {
        serde_json::json!({
            "access_count": *self.access_count.lock().unwrap(),
            "crash_every": CRASH_EVERY,
        })
    }
}
