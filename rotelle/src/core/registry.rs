use std::collections::HashMap;
use std::sync::Arc;
use super::Scenario;

pub type Factory = Box<dyn Fn() -> Arc<dyn Scenario> + Send + Sync>;

/// Maps scenario names to factory functions.
/// Every `create` call returns a fresh instance with no carried-over state.
pub struct ScenarioRegistry {
    entries: HashMap<&'static str, Factory>,
}

impl ScenarioRegistry {
    /// Build a registry from the list returned by [`crate::scenario::catalog::all`].
    /// Names are read from [`Scenario::name`] — never duplicated.
    pub fn from_catalog(factories: Vec<Factory>) -> Self {
        let mut entries = HashMap::new();
        for factory in factories {
            let name = factory().name();
            entries.insert(name, factory);
        }
        Self { entries }
    }

    /// Create a fresh instance of the named scenario, or `None` if unknown.
    pub fn create(&self, name: &str) -> Option<Arc<dyn Scenario>> {
        self.entries.get(name).map(|f| f())
    }
}
