use crate::core::{ActivationParams, ScenarioMeta, control_html};
use crate::state::{AppState, Mode};
use poem::{
    handler,
    web::{Data, Html},
};
use std::sync::atomic::Ordering;

#[handler]
pub async fn control(state: Data<&AppState>) -> Html<String> {
    if state.mode == Mode::Control {
        return control_proxy(&state).await;
    }

    let (active_name, active_desc) = {
        let scenario = state.active_scenario.lock().unwrap();
        (scenario.name(), scenario.description())
    };
    let active_params = state.active_params.lock().unwrap().clone();
    let sim_status = if state.crashed.load(Ordering::SeqCst) {
        "crashed".to_string()
    } else {
        "ok".to_string()
    };
    let scenarios = filtered_scenarios(&state);
    Html(control_html(
        active_name,
        active_desc,
        &active_params,
        &scenarios,
        &sim_status,
        &state.control_url,
        &state.sim_url,
    ))
}

/// Render the control page by fetching live state from the sim pod over HTTP.
async fn control_proxy(state: &AppState) -> Html<String> {
    let status = state.fetch_sim_status().await;

    let active_name = status["scenario"].as_str().unwrap_or("none, idle");
    let active_desc = status["description"].as_str().unwrap_or("");
    let active_params = ActivationParams::from_json(status["params"].clone());
    let sim_status = state.probe_sim_health().await;
    let scenarios = filtered_scenarios(state);

    Html(control_html(
        active_name,
        active_desc,
        &active_params,
        &scenarios,
        &sim_status,
        &state.control_url,
        &state.sim_url,
    ))
}

fn filtered_scenarios(state: &AppState) -> Vec<ScenarioMeta> {
    let mut scenarios = state.registry.list();
    scenarios.retain(|m| m.name != "check");
    scenarios.sort_by_key(|m| if m.name == "none, idle" { "" } else { m.name });
    scenarios
}
