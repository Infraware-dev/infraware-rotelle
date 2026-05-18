use crate::state::AppState;
use poem::{
    handler,
    web::{Data, Json},
};

#[handler]
pub fn status(state: Data<&AppState>) -> Json<serde_json::Value> {
    let scenario = state.active_scenario.lock().unwrap();
    let mut resp = serde_json::json!({
        "failure_case": scenario.name(),
        "description": scenario.description(),
    });
    if let serde_json::Value::Object(extras) = scenario.status_extras()
        && let serde_json::Value::Object(ref mut map) = resp
    {
        map.extend(extras);
    }
    Json(resp)
}
