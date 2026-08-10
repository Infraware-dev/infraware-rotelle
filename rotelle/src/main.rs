mod catalog;
mod core;
mod routes;
mod scenario;
mod state;

use core::registry::ScenarioRegistry;
use poem::{EndpointExt, Server, listener::TcpListener, middleware::AddData};
use scenario::idle::Idle;
use state::{AppState, Mode, load_state};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "/data".to_string());

    if !std::path::Path::new(&data_dir).is_dir() {
        error!(data_dir, "DATA_DIR does not exist or is not a directory");
        std::process::exit(1);
    }

    let mode = match std::env::var("ROTELLE_MODE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
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
    // shared with graceful shutdown failure: 0 = exit promptly on SIGTERM
    // N = ignore SIGTERM  for n seconds before existing
    let shutdown_armed = Arc::new(AtomicU64::new(0));
    let registry = ScenarioRegistry::from_catalog(catalog::all(Arc::clone(&shutdown_armed)));

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

    // Baseline: terminate promptly on SIGTERM. Required because rotelle runs as PID 1
    // in the scratch image — Linux discards SIGTERM for a PID namespace's init process
    // while the disposition is still SIG_DFL, so with no handler at all every pod
    // deletion waits out terminationGracePeriodSeconds and is then SIGKILLed.
    // graceful-shutdown-failure arms the flag below to make this path stall instead.
    {
        let armed = Arc::clone(&shutdown_armed);
        tokio::spawn(async move {
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("install SIGTERM handler");
            sigterm.recv().await;
            // The stream stays in scope for the whole window, so further SIGTERMs
            // are swallowed too — only SIGKILL can cut the ignore period short.
            match armed.load(Ordering::SeqCst) {
                0 => info!("SIGTERM received — shutting down"),
                secs => {
                    warn!(
                        ignore_secs = secs,
                        "graceful-shutdown-failure: SIGTERM ignored, still serving"
                    );
                    tokio::time::sleep(Duration::from_secs(secs)).await;
                    warn!("graceful-shutdown-failure: grace window elapsed, exiting");
                }
            }
            std::process::exit(0);
        });
    }

    let addr = format!("0.0.0.0:{port}");
    Server::new(TcpListener::bind(addr))
        .run(routes::routes(&mode).with(AddData::new(app_state)))
        .await
}
