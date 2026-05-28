use crate::core::page_html;
use crate::scenario::IndexEffect;
use crate::state::AppState;
use poem::{
    IntoResponse, Response, handler,
    http::StatusCode,
    web::{Data, Html},
};
use std::time::Duration;

#[handler]
pub async fn index(state: Data<&AppState>) -> Response {
    // Extract name, description, and effect while holding the lock, then drop it
    // before any await point — holding a MutexGuard across an await deadlocks.
    let (name, description, effect) = {
        let scenario = state.active_scenario.lock().unwrap();
        (
            scenario.name(),
            scenario.description(),
            scenario.on_index_request(),
        )
    };
    match effect {
        IndexEffect::Respond(body) => Html(page_html(name, description, &body)).into_response(),
        IndexEffect::Exit(code) => std::process::exit(code),
        IndexEffect::Hang => {
            tokio::time::sleep(Duration::from_secs(365 * 24 * 3600)).await;
            unreachable!()
        }
        IndexEffect::RespondWithStatus(code, html) => {
            let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (status, Html(html)).into_response()
        }
    }
}
