use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::Mutex;

const DEFAULT_REQUIRED_VAR: &str = "REQUIRED_APP_SECRET";

/// Simulates a deployment that exits on startup because a required env var is absent.
///
/// Activation just persists the config. The crash happens on the next pod restart
/// via `on_resume` (called before the server starts), which is the accurate
/// reproduction of a startup-check failure. Any GET / while active also exits,
/// so locally you can trigger the crash without restarting.
///
/// Setting the env var in the deployment "fixes" the scenario without a reset.
pub struct MissingEnvVarScenario {
    required_var: Mutex<String>,
}

impl MissingEnvVarScenario {
    pub fn new() -> Self {
        Self {
            required_var: Mutex::new(DEFAULT_REQUIRED_VAR.to_string()),
        }
    }

    fn var_name(&self) -> String {
        self.required_var.lock().unwrap().clone()
    }
}

impl Scenario for MissingEnvVarScenario {
    fn name(&self) -> &'static str {
        "missing-env-var"
    }

    fn description(&self) -> &'static str {
        "Exits if a required environment variable is absent — simulates a deployment missing required configuration."
    }

    fn activate(&self, params: &ActivationParams) {
        let var_name = params
            .get_string("required_var")
            .unwrap_or_else(|| DEFAULT_REQUIRED_VAR.to_string());
        *self.required_var.lock().unwrap() = var_name.clone();
        tracing::info!(var = %var_name, "missing-env-var: activated — crash will trigger on next pod restart or GET /");
    }

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        let var = self.var_name();
        if std::env::var(&var).is_err() {
            tracing::warn!(var = %var, "missing-env-var: env var absent, exiting");
            IndexEffect::Exit(1)
        } else {
            IndexEffect::Respond(format!(
                "<p>Environment variable <code>{var}</code> is present — \
                 startup check passed.</p>"
            ))
        }
    }

    /// Exit immediately if the required var is still absent — this is what produces
    /// CrashLoopBackOff. Called before the server starts, so the pod never becomes
    /// ready. If the operator has since added the var, resume normally.
    fn on_resume(&self, params: &ActivationParams) {
        let var_name = params
            .get_string("required_var")
            .unwrap_or_else(|| DEFAULT_REQUIRED_VAR.to_string());
        *self.required_var.lock().unwrap() = var_name.clone();

        if std::env::var(&var_name).is_err() {
            tracing::warn!(var = %var_name, "missing-env-var: required env var absent on pod restart, exiting");
            std::process::exit(1);
        }
        tracing::info!(var = %var_name, "missing-env-var: required env var present on resume");
    }

    fn status_extras(&self) -> serde_json::Value {
        let var = self.var_name();
        let present = std::env::var(&var).is_ok();
        serde_json::json!({
            "required_var": var,
            "var_present": present,
        })
    }
}
