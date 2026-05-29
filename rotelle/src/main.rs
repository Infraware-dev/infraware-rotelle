mod catalog;
mod core;
mod routes;
mod scenario;
mod state;

use core::registry::ScenarioRegistry;
use poem::{EndpointExt, Server, listener::TcpListener, middleware::AddData};
use scenario::idle::Idle;
use state::{AppState, Mode, load_state};
use std::sync::atomic::AtomicBool;
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

    let mode = match std::env::var("ROTELLE_MODE").unwrap_or_default().to_lowercase().as_str() {
        "control" => Mode::Control,
        _ => Mode::Full,
    };

    let port: u16 = std::env::var("ROTELLE_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(match mode {
            Mode::Control => 9090,
            _ => 8080,
        });

    let state_file = format!("{data_dir}/state.json");
    let registry = ScenarioRegistry::from_catalog(catalog::all());

    let (scenario, params) = load_state(&state_file);
    info!(data_dir, scenario = %scenario, port, ?mode, "rotelle starting");

    let initial = registry
        .create(&scenario)
        .unwrap_or_else(|| Arc::new(Idle::new()));

    // Only resume simulation in full mode — the control pod never runs scenarios.
    if mode != Mode::Control {
        initial.on_resume(&params);
    }

    let control_url =
        std::env::var("ROTELLE_CONTROL_URL").unwrap_or_else(|_| "/rotectl/control".to_string());
    let sim_url = std::env::var("ROTELLE_SIM_URL").unwrap_or_else(|_| "/".to_string());

    let sim_api_url = std::env::var("ROTELLE_SIM_API_URL").unwrap_or_default();
    if mode == Mode::Control && sim_api_url.is_empty() {
        error!(
            "ROTELLE_SIM_API_URL is not set — control mode will not be able to reach the sim pod"
        );
    }

    let app_state = AppState {
        active_scenario: Arc::new(Mutex::new(initial)),
        active_params: Arc::new(Mutex::new(params)),
        state_file: state_file.clone(),
        registry: Arc::new(registry),
        mode: mode.clone(),
        crashed: Arc::new(AtomicBool::new(false)),
        control_url,
        sim_url,
        sim_api_url,
        http_client: reqwest::Client::new(),
        last_known_sim_status: Arc::new(Mutex::new(serde_json::Value::Object(Default::default()))),
    };

    let addr = format!("0.0.0.0:{port}");
    Server::new(TcpListener::bind(addr))
        .run(routes::routes(&mode).with(AddData::new(app_state)))
        .await
}
