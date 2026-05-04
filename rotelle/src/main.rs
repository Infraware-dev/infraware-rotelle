mod routes;
mod scenario;
mod state;

use std::sync::{Arc, Mutex};
use scenario::catalog;
use scenario::idle::Idle;
use scenario::registry::ScenarioRegistry;
use state::{AppState, load_state};
use poem::{EndpointExt, Server, listener::TcpListener, middleware::AddData};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "/data".to_string());

    if !std::path::Path::new(&data_dir).is_dir() {
        error!(data_dir, "DATA_DIR does not exist or is not a directory");
        std::process::exit(1);
    }

    let state_file = format!("{data_dir}/state.json");
    let registry = ScenarioRegistry::from_catalog(catalog::all());

    let (case, params) = load_state(&state_file);
    info!(data_dir, failure_case = %case, "rotelle starting");

    let initial = registry
        .create(&case)
        .unwrap_or_else(|| Arc::new(Idle::new()));

    initial.on_resume(&params);

    let app_state = AppState {
        active_scenario: Arc::new(Mutex::new(initial)),
        state_file,
        registry: Arc::new(registry),
    };

    Server::new(TcpListener::bind("0.0.0.0:8080"))
        .run(routes::routes().with(AddData::new(app_state)))
        .await
}
