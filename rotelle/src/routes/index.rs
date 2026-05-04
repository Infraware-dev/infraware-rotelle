use crate::scenario::IndexEffect;
use crate::state::AppState;
use poem::{handler, web::{Data, Html}};

#[handler]
pub fn index(state: Data<&AppState>) -> Html<String> {
    match state.active_scenario.lock().unwrap().on_index_request() {
        IndexEffect::Respond(html) => Html(html),
        IndexEffect::Exit(code) => std::process::exit(code),
    }
}
