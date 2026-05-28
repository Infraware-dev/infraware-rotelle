use super::{ActivationParams, IndexEffect, Scenario};

pub struct ServiceUnreachableScenario;

impl ServiceUnreachableScenario {
    pub fn new() -> Self {
        Self
    }
}

impl Scenario for ServiceUnreachableScenario {
    fn name(&self) -> &'static str {
        "service-unreachable"
    }

    fn description(&self) -> &'static str {
        "Holds every GET / connection open indefinitely — simulates a Service selector mismatch where no traffic reaches the pod."
    }

    fn activate(&self, _: &ActivationParams) {}

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        tracing::info!("service-unreachable: hanging connection");
        IndexEffect::Hang
    }
}
