use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::Mutex;

const DEFAULT_DELAY_MS: u64 = 2000;

pub struct SlowResponseScenario {
    delay_ms: Mutex<u64>,
}

impl SlowResponseScenario {
    pub fn new() -> Self {
        Self {
            delay_ms: Mutex::new(DEFAULT_DELAY_MS),
        }
    }
}

impl Scenario for SlowResponseScenario {
    fn name(&self) -> &'static str {
        "slow-response"
    }

    fn description(&self) -> &'static str {
        "Delays every GET / response by a configurable duration — simulates a degraded pod whose slow responses exceed readiness probe timeouts."
    }

    fn activate(&self, params: &ActivationParams) {
        *self.delay_ms.lock().unwrap() = params.get_u64("delay_ms").unwrap_or(DEFAULT_DELAY_MS);
    }

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        let delay = *self.delay_ms.lock().unwrap();
        tracing::info!(delay_ms = delay, "slow-response: delaying index response");
        IndexEffect::RespondAfterDelay(
            delay,
            format!("<p>Responded after <strong>{delay} ms</strong></p>"),
        )
    }

    fn status_extras(&self) -> serde_json::Value {
        serde_json::json!({ "delay_ms": *self.delay_ms.lock().unwrap() })
    }

    fn default_params(&self) -> ActivationParams {
        ActivationParams::from_json(serde_json::json!({ "delay_ms": DEFAULT_DELAY_MS }))
    }
}
