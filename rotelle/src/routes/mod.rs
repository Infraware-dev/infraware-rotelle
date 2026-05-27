pub mod health;
pub mod index;
pub mod logo;
pub mod rotectl;

use crate::state::Mode;
use poem::{Route, get, post};

fn base() -> Route {
    Route::new()
        .at("/logo.png", get(logo::logo))
        .at("/favicon.png", get(logo::favicon))
}

fn with_sim(r: Route) -> Route {
    r.at("/", get(index::index))
        .at("/health", get(health::sim_health))
}

fn with_control(r: Route) -> Route {
    r.at("/rotectl/health", get(health::control_health))
        .at("/rotectl/status", get(rotectl::status::status))
        .at("/rotectl/cmd", post(rotectl::cmd::cmd))
        .at("/rotectl/control", get(rotectl::control::control))
        .at("/rotectl/sim-health", get(health::sim_health_json))
}

pub fn routes(mode: &Mode) -> Route {
    match mode {
        Mode::Sim => with_sim(base()),
        Mode::Control => with_control(base()),
        Mode::Full => with_control(with_sim(base())),
    }
}
