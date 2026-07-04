use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, Instant};

use super::types::LlmResponse;

struct CacheEntry {
    response: LlmResponse,
    created: Instant,
}

pub struct ResponseCache {
    entries: HashMap<u64, CacheEntry>,
    ttl: Duration,
}

impl ResponseCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self, system: &str, user: &str, context: &str) -> Option<&LlmResponse> {
        let key = Self::hash_key(system, user, context);
        let entry = self.entries.get(&key)?;
        if entry.created.elapsed() > self.ttl {
            return None;
        }
        Some(&entry.response)
    }

    pub fn put(&mut self, system: &str, user: &str, context: &str, response: LlmResponse) {
        let key = Self::hash_key(system, user, context);
        self.entries.insert(key, CacheEntry { response, created: Instant::now() });
    }

    fn hash_key(system: &str, user: &str, context: &str) -> u64 {
        let mut h = DefaultHasher::new();
        system.hash(&mut h);
        user.hash(&mut h);
        context.hash(&mut h);
        h.finish()
    }
}
