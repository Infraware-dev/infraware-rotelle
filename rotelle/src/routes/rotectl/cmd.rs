use crate::scenario::ActivationParams;
use crate::scenario::check::Check;
use crate::scenario::idle::Idle;
use crate::state::AppState;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

type CmdResponse = (StatusCode, Json<serde_json::Value>);

fn ok(scenario: &str) -> CmdResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "scenario": scenario})),
    )
}

fn err(msg: impl std::fmt::Display) -> CmdResponse {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"ok": false, "error": msg.to_string()})),
    )
}

#[derive(Deserialize)]
struct Cmd {
    cmd: String,
    scenario: Option<String>,
    /// All remaining JSON fields are collected as activation parameters.
    /// Scenarios read only the keys they recognise; unknown keys are ignored.
    #[serde(flatten)]
    params: HashMap<String, serde_json::Value>,
}

#[handler]
pub fn cmd(state: Data<&AppState>, Json(body): Json<Cmd>) -> CmdResponse {
    let params = ActivationParams::from(body.params);

    match body.cmd.as_str() {
        "set" => {
            let scenario = body.scenario.as_deref().unwrap_or("none, idle");
            match state.registry.create(scenario) {
                None => err(format!("unknown scenario: {scenario}")),
                Some(s) => {
                    let name = state.switch_scenario(s, params);
                    info!(scenario = name, "scenario set");
                    ok(name)
                }
            }
        }
        "reset" => {
            let name = state.switch_scenario(Arc::new(Idle::new()), ActivationParams::default());
            info!(scenario = name, "scenario reset");
            ok(name)
        }
        "check" => {
            let name = state.switch_scenario(Arc::new(Check::new()), ActivationParams::default());
            info!(scenario = name, "scenario set");
            ok(name)
        }
        unknown => err(format!("unknown cmd: {unknown}")),
    }
}
