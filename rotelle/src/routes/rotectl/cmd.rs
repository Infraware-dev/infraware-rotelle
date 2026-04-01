use poem::{handler, http::StatusCode, web::{Data, Json}};
use serde::Deserialize;
use tracing::info;
use crate::state::{persist_state, State};

#[derive(Deserialize)]
pub struct Cmd {
    pub cmd: String,
    pub case: Option<String>,
}

#[handler]
pub fn cmd(
    state: Data<&State>,
    Json(body): Json<Cmd>,
) -> (StatusCode, Json<serde_json::Value>) {
    match body.cmd.as_str() {
        "set" => {
            let case = body.case.unwrap_or_else(|| "none, idle".to_string());
            *state.failure_case.lock().unwrap() = case.clone();
            persist_state(&state.state_file, &case);
            info!(failure_case = case, "failure case set");
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "failure_case": case })))
        }
        "reset" => {
            *state.failure_case.lock().unwrap() = "none, idle".to_string();
            persist_state(&state.state_file, "none, idle");
            info!(failure_case = "none, idle", "failure case reset");
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "failure_case": "none, idle" })))
        }
        "check" => {
            let case = "check".to_string();
            *state.failure_case.lock().unwrap() = case.clone();
            persist_state(&state.state_file, &case);
            info!(failure_case = case, "failure case set");
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "failure_case": case })))
        }
        unknown => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "error": format!("unknown cmd: {unknown}") })),
        ),
    }
}
