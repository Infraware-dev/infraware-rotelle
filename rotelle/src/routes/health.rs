use crate::state::AppState;
use poem::{handler, http::StatusCode, web::{Data, Json}};
use std::sync::atomic::Ordering;

/// Simulation health — returns 503 when the active scenario has triggered a crash.
/// Kubernetes watches this endpoint; a 503 causes the pod to be restarted.
#[handler]
pub fn sim_health(state: Data<&AppState>) -> StatusCode {
    if state.crashed.load(Ordering::SeqCst) {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    }
}

/// Control-plane health — always returns 200.
/// The control plane (/rotectl/*) stays alive regardless of scenario state.
#[handler]
pub fn control_health() -> &'static str {
    "ok"
}

/// Returns simulation health as JSON for the control panel's JS health polling.
/// Status values: "ok", "crashed", "down" (OOMKilled / not running).
#[handler]
pub fn sim_health_json(state: Data<&AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": state.sim_health_status() }))
}
