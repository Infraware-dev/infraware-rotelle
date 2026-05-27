use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::AbortHandle;

pub struct OomKillScenario {
    memory_sink: Arc<Mutex<Vec<Vec<u8>>>>,
    leak_task: Mutex<Option<AbortHandle>>,
    loop_time_secs: Mutex<u64>,
    loop_amount_mb: Mutex<usize>,
}

impl OomKillScenario {
    pub fn new() -> Self {
        Self {
            memory_sink: Arc::new(Mutex::new(Vec::new())),
            leak_task: Mutex::new(None),
            loop_time_secs: Mutex::new(10),
            loop_amount_mb: Mutex::new(10),
        }
    }
}

impl Scenario for OomKillScenario {
    fn name(&self) -> &'static str {
        "oom-kill"
    }

    fn description(&self) -> &'static str {
        "Background task allocates memory on a timer until the pod is OOMKilled."
    }

    fn activate(&self, params: &ActivationParams) {
        let interval_secs = params.get_u64("loop_time_secs").unwrap_or(10);
        let amount_mb = params.get_usize("loop_amount_mb").unwrap_or(10);

        *self.loop_time_secs.lock().unwrap() = interval_secs;
        *self.loop_amount_mb.lock().unwrap() = amount_mb;

        let sink = Arc::clone(&self.memory_sink);
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
                let mut chunk = vec![0u8; amount_mb * 1024 * 1024];
                let noise = get_noise();
                for (i, page) in chunk.chunks_mut(4096).enumerate() {
                    page[0] = noise.wrapping_add(i as u8);
                }
                tracing::info!(amount_mb, "oom-kill: allocated memory chunk");
                sink.lock().unwrap().push(chunk);
            }
        })
        .abort_handle();

        *self.leak_task.lock().unwrap() = Some(handle);
    }

    fn deactivate(&self) {
        if let Some(handle) = self.leak_task.lock().unwrap().take() {
            handle.abort();
        }
        self.memory_sink.lock().unwrap().clear();
    }

    fn on_index_request(&self) -> IndexEffect {
        let chunks = self.memory_sink.lock().unwrap().len();
        IndexEffect::Respond(format!(
            "<p>Memory pressure events: <strong>{chunks}</strong></p>"
        ))
    }

    fn status_extras(&self) -> serde_json::Value {
        serde_json::json!({
            "loop_time_secs": *self.loop_time_secs.lock().unwrap(),
            "loop_amount_mb": *self.loop_amount_mb.lock().unwrap(),
        })
    }

    fn default_params(&self) -> ActivationParams {
        ActivationParams::from_json(serde_json::json!({
            "loop_time_secs": 10,
            "loop_amount_mb": 10
        }))
    }
}

fn get_noise() -> u8 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u8)
        .unwrap_or(0xAB)
}
