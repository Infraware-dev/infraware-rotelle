use std::sync::Arc;

use super::check::Check;
use super::idle::Idle;
use super::ingress_conflict::IngressConflictScenario;
use super::crash_loop::CrashLoopScenario;
use super::oom_kill::OomKillScenario;
use super::missing_env_var::MissingEnvVarScenario;
use super::service_unreachable::ServiceUnreachableScenario;
use crate::core::registry::Factory;

/// All built-in failure scenarios.
///
/// **To add a new scenario: add one line here.**
/// The name is read automatically from `Scenario::name()`.
pub fn all() -> Vec<Factory> {
    vec![
        Box::new(|| Arc::new(Idle::new())),
        Box::new(|| Arc::new(Check::new())),
        Box::new(|| Arc::new(CrashLoopScenario::new())),
        Box::new(|| Arc::new(OomKillScenario::new())),
        Box::new(|| Arc::new(MissingEnvVarScenario::new())),
        Box::new(|| Arc::new(ServiceUnreachableScenario::new())),
        Box::new(|| Arc::new(IngressConflictScenario::new())),
    ]
}
