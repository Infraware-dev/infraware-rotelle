use super::{Scenario, ScenarioMeta};
use std::collections::HashMap;
use std::sync::Arc;

pub type Factory = Box<dyn Fn() -> Arc<dyn Scenario> + Send + Sync>;

/// Maps scenario names to factory functions.
/// Every `create` call returns a fresh instance with no carried-over state.
pub struct ScenarioRegistry {
    entries: HashMap<&'static str, Factory>,
}

impl ScenarioRegistry {
    /// Names are read from [`Scenario::name`]. If two factories return the same name the
    /// second silently wins, so keep names unique across `catalog::all()`.
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

    /// Returns [`ScenarioMeta`] for every registered scenario, sorted by name.
    pub fn list(&self) -> Vec<ScenarioMeta> {
        let mut entries: Vec<_> = self
            .entries
            .values()
            .map(|f| {
                let s = f();
                ScenarioMeta {
                    name: s.name(),
                    description: s.description(),
                }
            })
            .collect();
        entries.sort_by_key(|m| m.name);
        entries
    }
}
