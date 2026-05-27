use crate::core::control_html;
use crate::state::AppState;
use poem::{
    handler,
    web::{Data, Html},
};

#[handler]
pub fn control(state: Data<&AppState>) -> Html<String> {
    let (active_name, active_desc) = {
        let scenario = state.active_scenario.lock().unwrap();
        (scenario.name(), scenario.description())
    };
    let active_params = state.active_params.lock().unwrap().clone();
    let sim_status = state.sim_health_status();
    let mut scenarios = state.registry.list();
    // "check" is an internal test-sequence marker, not a user-facing scenario.
    scenarios.retain(|m| m.name != "check");
    // "none, idle" (reset state) always appears first.
    scenarios.sort_by_key(|m| if m.name == "none, idle" { "" } else { m.name });
    Html(control_html(active_name, active_desc, &active_params, &scenarios, &sim_status, &state.control_url, &state.sim_url))
}
