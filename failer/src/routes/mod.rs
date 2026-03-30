pub mod failctl;
pub mod health;
pub mod index;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/", get(index::index))
        .at("/health", get(health::health))
        .at("/failctl/health", get(failctl::health::health))
        .at("/failctl/status", get(failctl::status::status))
        .at("/failctl/cmd", post(failctl::cmd::cmd))
}
