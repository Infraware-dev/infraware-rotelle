use crate::scenario::IndexEffect;
use crate::state::AppState;
use poem::{IntoResponse, Response, handler, http::StatusCode, web::{Data, Html}};
use std::time::Duration;

#[handler]
pub async fn index(state: Data<&AppState>) -> Response {
    // Extract the effect before the match so the MutexGuard is dropped before
    // any await point — holding it across an await would deadlock other requests.
    let effect = state.active_scenario.lock().unwrap().on_index_request();
    match effect {
        IndexEffect::Respond(html) => Html(html).into_response(),
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
