use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

const DEFAULT_HOLD_MS: u64 = 500;
const DEFAULT_MAX_CONCURRENT: usize = 3;

pub struct ThreadExhaustionScenario {
    hold_ms: Mutex<u64>,
    max_concurrent: Mutex<usize>,
    /// Rebuilt on every activate — a Semaphore's permit count is fixed at
    /// construction, and max_concurrent is an activation parameter.
    gate: Mutex<Arc<Semaphore>>,
    total_requests: AtomicU64,
}

impl ThreadExhaustionScenario {
    pub fn new() -> Self {
        Self {
            hold_ms: Mutex::new(DEFAULT_HOLD_MS),
            max_concurrent: Mutex::new(DEFAULT_MAX_CONCURRENT),
            gate: Mutex::new(Arc::new(Semaphore::new(DEFAULT_MAX_CONCURRENT))),
            total_requests: AtomicU64::new(0),
        }
    }
}

impl Scenario for ThreadExhaustionScenario {
    fn name(&self) -> &'static str {
        "thread-exhaustion"
    }

    fn description(&self) -> &'static str {
        "Serves only max_concurrent requests at a time, each holding its slot for hold_ms — \
         simulates a pod whose request handlers are all blocked on a slow downstream \
         dependency, so latency grows with load while the pod stays alive and probes pass."
    }

    fn activate(&self, params: &ActivationParams) {
        let hold_ms = params.get_u64("hold_ms").unwrap_or(DEFAULT_HOLD_MS);
        // Zero permits would park every request forever with no observable queue —
        // clamp to 1 so the scenario stays diagnosable under load.
        let max_concurrent = params
            .get_usize("max_concurrent")
            .unwrap_or(DEFAULT_MAX_CONCURRENT)
            .max(1);

        *self.hold_ms.lock().unwrap() = hold_ms;
        *self.max_concurrent.lock().unwrap() = max_concurrent;
        *self.gate.lock().unwrap() = Arc::new(Semaphore::new(max_concurrent));
        self.total_requests.store(0, Ordering::Relaxed);

        tracing::info!(hold_ms, max_concurrent, "thread-exhaustion: gate armed");
    }

    fn deactivate(&self) {
        // Closing the gate fails queued acquires immediately, so a reset drains the
        // backlog instead of leaving it to trickle out hold_ms at a time.
        self.gate.lock().unwrap().close();
        self.total_requests.store(0, Ordering::Relaxed);
    }

    fn on_index_request(&self) -> IndexEffect {
        let gate = {
            let guard = self.gate.lock().unwrap();
            Arc::clone(&guard)
        };
        let hold_ms = *self.hold_ms.lock().unwrap();
        let n = self.total_requests.fetch_add(1, Ordering::Relaxed) + 1;

        tracing::info!(
            request = n,
            slots_free = gate.available_permits(),
            "thread-exhaustion: request entering the gate"
        );

        IndexEffect::QueueBehindGate { gate, hold_ms }
    }

    fn status_extras(&self) -> serde_json::Value {
        let max_concurrent = *self.max_concurrent.lock().unwrap();
        let free = self.gate.lock().unwrap().available_permits();
        serde_json::json!({
            "hold_ms": *self.hold_ms.lock().unwrap(),
            "max_concurrent": max_concurrent,
            // Pegged at max_concurrent under load — the signal that every handler is
            // occupied and new requests are queueing rather than being served.
            "slots_in_use": max_concurrent.saturating_sub(free),
            "total_requests": self.total_requests.load(Ordering::Relaxed),
        })
    }

    fn default_params(&self) -> ActivationParams {
        ActivationParams::from_json(serde_json::json!({
            "hold_ms": DEFAULT_HOLD_MS,
            "max_concurrent": DEFAULT_MAX_CONCURRENT
        }))
    }
}
