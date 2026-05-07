use std::sync::Mutex;
use super::{ActivationParams, IndexEffect, Scenario};

const CRASH_EVERY: u32 = 5;

/// Simulates intermittent pod crashes: exits on every N-th GET /.
///
/// Kubernetes detects the non-zero exit code and restarts the pod.
pub struct Intermittent01Scenario {
    access_count: Mutex<u32>,
}

impl Intermittent01Scenario {
    pub fn new() -> Self {
        Self {
            access_count: Mutex::new(0),
        }
    }
}

impl Scenario for Intermittent01Scenario {
    fn name(&self) -> &'static str {
        "intermittent-01"
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
        tracing::info!(count = n, "intermittent-01: index access");
        if n % CRASH_EVERY == 0 {
            tracing::warn!(count = n, "intermittent-01: simulating crash");
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
