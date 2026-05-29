use crate::state::{AppState, Mode};
use poem::{
    handler,
    web::{Data, Json},
};

#[handler]
pub async fn status(state: Data<&AppState>) -> Json<serde_json::Value> {
    if state.mode == Mode::Control {
        return Json(state.fetch_sim_status().await);
    }

    let (name, desc, extras) = {
        let s = state.active_scenario.lock().unwrap();
        (s.name(), s.description(), s.status_extras())
    };
    let params = state.active_params.lock().unwrap().clone();
    let mut resp = serde_json::json!({
        "scenario": name,
        "description": desc,
        "params": params,
    });
    if let serde_json::Value::Object(extras_map) = extras
        && let serde_json::Value::Object(ref mut map) = resp
    {
        map.extend(extras_map);
    }
    Json(resp)
}
