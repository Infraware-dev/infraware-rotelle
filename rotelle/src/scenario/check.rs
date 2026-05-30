use super::{ActivationParams, IndexEffect, Scenario};

pub struct Check;

impl Check {
    pub fn new() -> Self {
        Self
    }
}

impl Scenario for Check {
    fn name(&self) -> &'static str {
        "check"
    }

    fn description(&self) -> &'static str {
        "Control checkpoint — marks a known state for test verification."
    }

    fn activate(&self, _: &ActivationParams) {}

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        IndexEffect::Respond(String::new())
    }
}
