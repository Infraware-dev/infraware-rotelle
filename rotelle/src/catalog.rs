use std::sync::Arc;

use crate::core::registry::Factory;
use crate::scenario::check::Check;
use crate::scenario::config_stale::ConfigStaleScenario;
use crate::scenario::crash_loop::CrashLoopScenario;
use crate::scenario::idle::Idle;
use crate::scenario::ingress_conflict::IngressConflictScenario;
use crate::scenario::keep_alive_timeout::KeepAliveTimeoutScenario;
use crate::scenario::missing_env_var::MissingEnvVarScenario;
use crate::scenario::oom_kill::OomKillScenario;
use crate::scenario::service_unreachable::ServiceUnreachableScenario;
use crate::scenario::slow_response::SlowResponseScenario;

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
        Box::new(|| Arc::new(KeepAliveTimeoutScenario::new())),
        Box::new(|| Arc::new(IngressConflictScenario::new())),
        Box::new(|| Arc::new(ConfigStaleScenario::new())),
        Box::new(|| Arc::new(SlowResponseScenario::new())),
    ]
}
