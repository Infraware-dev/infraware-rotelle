use crate::core::page_html;
use crate::scenario::IndexEffect;
use crate::state::AppState;
use poem::{
    IntoResponse, Response, handler,
    http::StatusCode,
    web::{Data, Html},
};
use std::time::{Duration, Instant};

#[handler]
pub async fn index(state: Data<&AppState>) -> Response {
    // Extract only the effect while holding the lock, then drop it before any
    // await point — holding a MutexGuard across an await deadlocks.
    let (name, effect) = {
        let scenario = state.active_scenario.lock().unwrap();
        (scenario.name(), scenario.on_index_request())
    };
    match effect {
        IndexEffect::Respond(body) => {
            Html(page_html(name, &body, &state.control_url, &state.sim_url)).into_response()
        }
        IndexEffect::Exit => {
            // Always exit the process — Kubernetes applies real CrashLoopBackOff backoff
            // (10 s → 20 s → 40 s … up to 5 min). The separate control pod in
            // rotelle-system survives and shows the "down" badge while the sim restarts.
            // In local dev (just run, no control pod), restart the server manually.
            std::process::exit(1);
        }
        IndexEffect::Hang => {
            tokio::time::sleep(Duration::from_secs(365 * 24 * 3600)).await;
            unreachable!()
        }
        IndexEffect::RespondWithStatus(code, html) => {
            let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (status, Html(html)).into_response()
        }
        IndexEffect::RespondThenClose(body) => {
            let mut response =
                Html(page_html(name, &body, &state.control_url, &state.sim_url)).into_response();
            response.headers_mut().insert(
                poem::http::header::CONNECTION,
                poem::http::HeaderValue::from_static("close"),
            );
            response
        }
        IndexEffect::RespondAfterDelay(delay_ms, body) => {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            Html(page_html(name, &body, &state.control_url, &state.sim_url)).into_response()
        }
        IndexEffect::QueueBehindGate { gate, hold_ms } => {
            let queued_at = Instant::now();
            // Waiting here rather than in on_index_request keeps the scenario mutex
            // free, so /rotectl/* stays instant while requests pile up behind the gate.
            let permit = match gate.acquire().await {
                Ok(permit) => permit,
                // deactivate() closes the gate — drain the backlog immediately instead
                // of serving a scenario that is no longer active.
                Err(_) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Html("<p>Scenario reset while this request was queued.</p>"),
                    )
                        .into_response();
                }
            };
            let waited_ms = queued_at.elapsed().as_millis();
            tokio::time::sleep(Duration::from_millis(hold_ms)).await;
            drop(permit);
            let body = format!(
                "<p>Queued <strong>{waited_ms} ms</strong> for a free slot, \
                 then held it for <strong>{hold_ms} ms</strong>.</p>"
            );
            Html(page_html(name, &body, &state.control_url, &state.sim_url)).into_response()
        }
    }
}
