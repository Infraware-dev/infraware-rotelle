pub mod registry;
pub mod scenario_template;

pub use scenario_template::{ActivationParams, IndexEffect, Scenario, control_html, page_html};
pub(crate) use scenario_template::ScenarioMeta;
