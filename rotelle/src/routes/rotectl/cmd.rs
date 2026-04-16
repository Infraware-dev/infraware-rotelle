use crate::state::{State, persist_state};
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize)]
pub struct Cmd {
    pub cmd: String,
    pub case: Option<String>,
    /// intermittent-02: seconds between memory allocations (default: 10)
    pub loop_time_secs: Option<u64>,
    /// intermittent-02: megabytes to allocate per loop iteration (default: 10)
    pub loop_amount_mb: Option<usize>,
}

#[handler]
pub fn cmd(state: Data<&State>, Json(body): Json<Cmd>) -> (StatusCode, Json<serde_json::Value>) {
    match body.cmd.as_str() {
        "set" => {
            let case = body.case.unwrap_or_else(|| "none, idle".to_string());
            state.reset_behavioral();
            if case == "intermittent-02" {
                start_leak_task(&state, body.loop_time_secs, body.loop_amount_mb);
            }
            *state.failure_case.lock().unwrap() = case.clone();
            persist_state(&state.state_file, &case);
            info!(failure_case = case, "failure case set");
            (
                StatusCode::OK,
                Json(serde_json::json!({ "ok": true, "failure_case": case })),
            )
        }
        "reset" => {
            state.reset_behavioral();
            *state.failure_case.lock().unwrap() = "none, idle".to_string();
            persist_state(&state.state_file, "none, idle");
            info!(failure_case = "none, idle", "failure case reset");
            (
                StatusCode::OK,
                Json(serde_json::json!({ "ok": true, "failure_case": "none, idle" })),
            )
        }
        "check" => {
            state.reset_behavioral();
            let case = "check".to_string();
            *state.failure_case.lock().unwrap() = case.clone();
            persist_state(&state.state_file, &case);
            info!(failure_case = case, "failure case set");
            (
                StatusCode::OK,
                Json(serde_json::json!({ "ok": true, "failure_case": case })),
            )
        }
        unknown => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "error": format!("unknown cmd: {unknown}") })),
        ),
    }
}

fn get_noise() -> u8 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u8)
        .unwrap_or(0xAB)
}

fn start_leak_task(state: &State, loop_time_secs: Option<u64>, loop_amount_mb: Option<usize>) {
    let interval_secs = loop_time_secs.unwrap_or(10);
    let amount_mb = loop_amount_mb.unwrap_or(10);
    let sink = Arc::clone(&state.memory_sink);

    let abort = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
            let mut chunk = vec![0u8; amount_mb * 1024 * 1024];
            let noise = get_noise();
            for (i, page) in chunk.chunks_mut(4096).enumerate() {
                page[0] = noise.wrapping_add(i as u8);
            }
            info!(amount_mb, "intermittent-02: allocated memory chunk");
            sink.lock().unwrap().push(chunk);
        }
    })
    .abort_handle();

    *state.leak_task.lock().unwrap() = Some(abort);
}
