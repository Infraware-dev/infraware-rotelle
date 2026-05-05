use std::sync::Arc;

use super::check::Check;
use super::idle::Idle;
use super::intermittent_01::Intermittent01Scenario;
use super::intermittent_02::Intermittent02Scenario;
use super::missing_env_var::MissingEnvVarScenario;
use super::ingress_conflict::IngressConflictScenario;
use super::service_unreachable::ServiceUnreachableScenario;
use super::registry::Factory;

/// All built-in failure scenarios.
///
/// **To add a new scenario: add one line here.**
/// The name is read automatically from `Scenario::name()`.
pub fn all() -> Vec<Factory> {
    vec![
        Box::new(|| Arc::new(Idle::new())),
        Box::new(|| Arc::new(Check::new())),
        Box::new(|| Arc::new(Intermittent01Scenario::new())),
        Box::new(|| Arc::new(Intermittent02Scenario::new())),
        Box::new(|| Arc::new(MissingEnvVarScenario::new())),
        Box::new(|| Arc::new(ServiceUnreachableScenario::new())),
        Box::new(|| Arc::new(IngressConflictScenario::new())),
    ]
}
