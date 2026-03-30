mod routes;
mod state;

use poem::{EndpointExt, Server, listener::TcpListener, middleware::AddData};
use state::{State, load_state};
use std::sync::{Arc, Mutex};
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
    let initial = load_state(&state_file);

    info!(data_dir, failure_case = initial, "failer starting");

    let state = State {
        failure_case: Arc::new(Mutex::new(initial)),
        state_file,
    };

    Server::new(TcpListener::bind("0.0.0.0:8080"))
        .run(routes::routes().with(AddData::new(state)))
        .await
}
