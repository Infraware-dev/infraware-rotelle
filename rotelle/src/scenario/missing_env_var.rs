use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::Mutex;

const DEFAULT_REQUIRED_VAR: &str = "REQUIRED_APP_SECRET";

/// Simulates a deployment that exits on startup because a required env var is absent.
///
/// Two parameters control the scenario entirely from the control panel:
/// - `required_var` — name of the env var being checked (display / docs only)
/// - `var_value`    — leave EMPTY to simulate the var being absent (→ crash);
///   fill in ANY text to simulate the var being present (→ pass)
pub struct MissingEnvVarScenario {
    required_var: Mutex<String>,
    var_value: Mutex<String>,
}

impl MissingEnvVarScenario {
    pub fn new() -> Self {
        Self {
            required_var: Mutex::new(DEFAULT_REQUIRED_VAR.to_string()),
            var_value: Mutex::new(String::new()),
        }
    }
}

impl Scenario for MissingEnvVarScenario {
    fn name(&self) -> &'static str {
        "missing-env-var"
    }

    fn description(&self) -> &'static str {
        "Simulates a missing required env var. Leave var_value empty to trigger the crash; set it to any value to simulate the var being present."
    }

    fn activate(&self, params: &ActivationParams) {
        let var_name = params
            .get_string("required_var")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_REQUIRED_VAR.to_string());
        let var_value = params.get_string("var_value").unwrap_or_default();
        *self.required_var.lock().unwrap() = var_name.clone();
        *self.var_value.lock().unwrap() = var_value.clone();
        tracing::info!(var = %var_name, present = !var_value.is_empty(), "missing-env-var: activated");
    }

    fn deactivate(&self) {
        *self.var_value.lock().unwrap() = String::new();
    }

    fn on_index_request(&self) -> IndexEffect {
        let var = self.required_var.lock().unwrap().clone();
        let present = !self.var_value.lock().unwrap().is_empty();
        if present {
            IndexEffect::Respond(format!(
                "<p>Environment variable <code>{var}</code> is present — startup check passed.</p>"
            ))
        } else {
            tracing::warn!(var = %var, "missing-env-var: env var absent, exiting");
            IndexEffect::Exit
        }
    }

    fn status_extras(&self) -> serde_json::Value {
        let var = self.required_var.lock().unwrap().clone();
        let present = !self.var_value.lock().unwrap().is_empty();
        serde_json::json!({ "required_var": var, "var_present": present })
    }

    fn default_params(&self) -> ActivationParams {
        ActivationParams::from_json(serde_json::json!({
            "required_var": "REQUIRED_APP_SECRET",
            "var_value": ""
        }))
    }
}
