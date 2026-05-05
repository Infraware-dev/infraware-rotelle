use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::AbortHandle;
use super::{ActivationParams, IndexEffect, Scenario};

/// Simulates memory exhaustion: background task leaks memory until OOMKill.
pub struct Intermittent02Scenario {
    memory_sink: Arc<Mutex<Vec<Vec<u8>>>>,
    leak_task: Mutex<Option<AbortHandle>>,
    active_params: Mutex<Option<ActivationParams>>,
}

impl Intermittent02Scenario {
    pub fn new() -> Self {
        Self {
            memory_sink: Arc::new(Mutex::new(Vec::new())),
            leak_task: Mutex::new(None),
            active_params: Mutex::new(None),
        }
    }
}

impl Scenario for Intermittent02Scenario {
    fn name(&self) -> &'static str {
        "intermittent-02"
    }

    fn description(&self) -> &'static str {
        "Background task allocates memory on a timer until the pod is OOMKilled."
    }

    fn activate(&self, params: &ActivationParams) {
        let interval_secs = params.get_u64("loop_time_secs").unwrap_or(10);
        let amount_mb = params.get_usize("loop_amount_mb").unwrap_or(10);

        *self.active_params.lock().unwrap() = Some(params.clone());

        let sink = Arc::clone(&self.memory_sink);
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
                let mut chunk = vec![0u8; amount_mb * 1024 * 1024];
                let noise = get_noise();
                for (i, page) in chunk.chunks_mut(4096).enumerate() {
                    page[0] = noise.wrapping_add(i as u8);
                }
                tracing::info!(amount_mb, "intermittent-02: allocated memory chunk");
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
        *self.active_params.lock().unwrap() = None;
    }

    fn on_index_request(&self) -> IndexEffect {
        let chunks = self.memory_sink.lock().unwrap().len();
        IndexEffect::Respond(format!(
            "<p>Allocated chunks so far: <strong>{chunks}</strong></p>"
        ))
    }

    fn status_extras(&self) -> serde_json::Value {
        match &*self.active_params.lock().unwrap() {
            Some(params) => params.to_json(),
            None => serde_json::Value::Null,
        }
    }
}

fn get_noise() -> u8 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u8)
        .unwrap_or(0xAB)
}
