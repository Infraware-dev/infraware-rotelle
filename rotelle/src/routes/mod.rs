pub mod rotectl;
pub mod health;
pub mod index;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/", get(index::index))
        .at("/health", get(health::health))
        .at("/rotectl/health", get(rotectl::health::health))
        .at("/rotectl/status", get(rotectl::status::status))
        .at("/rotectl/cmd", post(rotectl::cmd::cmd))
}
