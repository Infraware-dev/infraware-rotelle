use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

const DEFAULT_IGNORE_SECS: u64 = 30;

/// Simulates a pod that ignores SIGTERM during a rolling update.
///
/// While active, the process-wide SIGTERM handler installed in `main` stalls for
/// `ignore_secs` instead of exiting, and `GET /` keeps returning 200 the whole time —
/// so the pod stays in Endpoints, keeps taking traffic, and its in-flight requests are
/// severed when the kubelet's SIGKILL lands.
pub struct GracefulShutdownFailureScenario {
    ignore_secs: Mutex<u64>,
    /// Shared with the SIGTERM handler in `main`: 0 = exit promptly, N = ignore for N secs.
    armed: Arc<AtomicU64>,
    requests_served: Arc<AtomicU64>,
}

impl GracefulShutdownFailureScenario {
    pub fn new(armed: Arc<AtomicU64>) -> Self {
        Self {
            ignore_secs: Mutex::new(DEFAULT_IGNORE_SECS),
            armed,
            requests_served: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Scenario for GracefulShutdownFailureScenario {
    fn name(&self) -> &'static str {
        "graceful-shutdown-failure"
    }

    fn description(&self) -> &'static str {
        "Ignores SIGTERM for a configurable window while still serving traffic — \
         reproduces dropped connections during rolling updates caused by a missing \
         preStop hook or improper shutdown handling."
    }

    fn activate(&self, params: &ActivationParams) {
        let ignore_secs = params.get_u64("ignore_secs").unwrap_or(DEFAULT_IGNORE_SECS);
        *self.ignore_secs.lock().unwrap() = ignore_secs;
        self.armed.store(ignore_secs, Ordering::SeqCst);
        tracing::info!(ignore_secs, "graceful-shutdown-failure: armed");
    }

    fn deactivate(&self) {
        // Disarm only — the signal stream in `main` must stay alive. Tearing it down
        // would leave SIGTERM permanently non-default, and as PID 1 in the scratch
        // image the pod would then only ever die by SIGKILL.
        self.armed.store(0, Ordering::SeqCst);
        self.requests_served.store(0, Ordering::Relaxed);
    }

    fn on_index_request(&self) -> IndexEffect {
        let n = self.requests_served.fetch_add(1, Ordering::Relaxed) + 1;
        let ignore_secs = *self.ignore_secs.lock().unwrap();
        IndexEffect::Respond(format!(
            "<p>Serving normally. On SIGTERM this pod will keep answering for \
             <strong>{ignore_secs}s</strong> before exiting.</p>\
             <p>Requests served: <strong>{n}</strong></p>"
        ))
    }

    fn status_extras(&self) -> serde_json::Value {
        serde_json::json!({
            "ignore_secs": *self.ignore_secs.lock().unwrap(),
            "armed": self.armed.load(Ordering::SeqCst) > 0,
            "requests_served": self.requests_served.load(Ordering::Relaxed),
        })
    }

    fn default_params(&self) -> ActivationParams {
        ActivationParams::from_json(serde_json::json!({ "ignore_secs": 30 }))
    }
}
