pub mod health;
pub mod index;
pub mod logo;
pub mod rotectl;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/", get(index::index))
        .at("/logo.png", get(logo::logo))
        .at("/favicon.png", get(logo::favicon))
        .at("/health", get(health::health))
        .at("/rotectl/health", get(health::health))
        .at("/rotectl/status", get(rotectl::status::status))
        .at("/rotectl/cmd", post(rotectl::cmd::cmd))
}
