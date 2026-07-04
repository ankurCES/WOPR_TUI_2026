use std::collections::HashMap;

use super::types::Scenario;

pub struct PrefetchCache {
    cache: HashMap<u32, Scenario>,
}

impl PrefetchCache {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn store(&mut self, option_id: u32, scenario: Scenario) {
        self.cache.insert(option_id, scenario);
    }

    pub fn take(&mut self, option_id: u32) -> Option<Scenario> {
        self.cache.remove(&option_id)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn has(&self, option_id: u32) -> bool {
        self.cache.contains_key(&option_id)
    }
}
