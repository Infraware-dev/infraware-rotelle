use crate::core::page_html;
use crate::scenario::IndexEffect;
use crate::state::{AppState, Mode};
use poem::{
    IntoResponse, Response, handler,
    http::StatusCode,
    web::{Data, Html},
};
use std::sync::atomic::Ordering;
use std::time::Duration;

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
            // Hide the Control nav link in sim mode — the control panel is on a separate
            // port and must not be discoverable from the simulation surface.
            let control_url = if state.mode == Mode::Sim { "" } else { &state.control_url };
            Html(page_html(name, &body, control_url, &state.sim_url)).into_response()
        }
        IndexEffect::Exit => {
            // In sim mode the control container is a separate process, so we can exit
            // for real — Kubernetes applies genuine CrashLoopBackOff (10s → 20s → 40s …
            // up to 5 min). Recovery works via state.json: the user sets a fixing param
            // in the control panel, and the next restart picks it up.
            // In full mode (single process, local dev) we can't exit, so fall back to
            // the probe-failure approach.
            state.write_sim_health("crashed");
            if state.mode == Mode::Sim {
                std::process::exit(1);
            }
            state.crashed.store(true, Ordering::SeqCst);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
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
