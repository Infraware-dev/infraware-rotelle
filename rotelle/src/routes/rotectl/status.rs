use crate::state::State;
use poem::{
    handler,
    web::{Data, Json},
};

#[handler]
pub fn status(state: Data<&State>) -> Json<serde_json::Value> {
    let case = state.failure_case.lock().unwrap().clone();
    Json(serde_json::json!({ "failure_case": case }))
}
