use std::sync::Arc;
use super::check::Check;
use super::idle::Idle;
use super::intermittent_01::Intermittent01Scenario;
use super::intermittent_02::Intermittent02Scenario;
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
    ]
}
