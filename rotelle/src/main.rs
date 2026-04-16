mod routes;
mod state;

use poem::{EndpointExt, Server, listener::TcpListener, middleware::AddData};
use state::{State, load_state};
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use crate::routes::rotectl::cmd::start_leak_task;

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

    let is_intermittent_02: bool = initial == "intermittent-02";

    info!(data_dir, failure_case = initial, "rotelle starting");

    let state = State {
        failure_case: Arc::new(Mutex::new(initial)),
        state_file,
        access_count: Arc::new(Mutex::new(0)),
        memory_sink: Arc::new(Mutex::new(Vec::new())),
        leak_task: Arc::new(Mutex::new(None)),
    };

    // Restart memleak after pod-restart
    if is_intermittent_02 {
        info!("Resuming intermittent-02 memory leak task...");
        start_leak_task(&state, None, None);
    }

    Server::new(TcpListener::bind("0.0.0.0:8080"))
        .run(routes::routes().with(AddData::new(state)))
        .await
}
