use super::{ActivationParams, IndexEffect, Scenario};

pub struct Idle;

impl Idle {
    pub fn new() -> Self { Self }
}

impl Scenario for Idle {
    fn name(&self) -> &'static str { "none, idle" }

    fn description(&self) -> &'static str {
        "No failure active — service responds normally."
    }

    fn activate(&self, _: &ActivationParams) {}

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        IndexEffect::Respond(String::new())
    }
}
