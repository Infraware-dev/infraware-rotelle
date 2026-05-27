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
        "sim" => Mode::Sim,
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

    // Only call on_resume in full/sim mode — the control container never runs the simulation.
    if mode != Mode::Control {
        initial.on_resume(&params);
    }

    let control_url =
        std::env::var("ROTELLE_CONTROL_URL").unwrap_or_else(|_| "/rotectl/control".to_string());
    let sim_url = std::env::var("ROTELLE_SIM_URL").unwrap_or_else(|_| "/".to_string());

    let app_state = AppState {
        active_scenario: Arc::new(Mutex::new(initial)),
        active_params: Arc::new(Mutex::new(params)),
        data_dir: data_dir.clone(),
        state_file: state_file.clone(),
        registry: Arc::new(registry),
        mode: mode.clone(),
        crashed: Arc::new(AtomicBool::new(false)),
        control_url,
        sim_url,
    };

    // ── Background tasks ─────────────────────────────────────────────────────

    if mode == Mode::Sim {
        // Poll state.json every 2 s and apply changes made by the control container.
        {
            let state = app_state.clone();
            let path = state_file.clone();
            tokio::spawn(async move {
                let mut last = std::fs::read_to_string(&path).unwrap_or_default();
                loop {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    let current = match std::fs::read_to_string(&path) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    if current == last {
                        continue;
                    }
                    last = current;
                    let (new_scenario, new_params) = load_state(&path);
                    if let Some(s) = state.registry.create(&new_scenario) {
                        info!(
                            scenario = new_scenario,
                            "sim: detected state change, applying"
                        );
                        state.apply_scenario(s, new_params);
                    }
                }
            });
        }

        // Write sim_health file every 2 s as a heartbeat so the control sidecar can
        // detect when this container goes down (stale file → "down" after 30 s).
        // Crashes and recoveries are also written immediately via write_sim_health().
        {
            let state = app_state.clone();
            let health_path = format!("{data_dir}/sim_health");
            tokio::spawn(async move {
                // Wait before the first write so a "crashed" file left by the previous
                // process stays visible long enough for the control panel badge to catch
                // it — even on a near-instant first CrashLoopBackOff restart.
                tokio::time::sleep(Duration::from_secs(4)).await;
                loop {
                    use std::sync::atomic::Ordering;
                    let status = if state.crashed.load(Ordering::SeqCst) {
                        "crashed"
                    } else {
                        "ok"
                    };
                    if let Err(e) = std::fs::write(&health_path, status) {
                        warn!(error = %e, "sim: failed to write sim_health file");
                    }
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            });
        }
    }

    // ── Server ───────────────────────────────────────────────────────────────

    let addr = format!("0.0.0.0:{port}");
    Server::new(TcpListener::bind(addr))
        .run(routes::routes(&mode).with(AddData::new(app_state)))
        .await
}
