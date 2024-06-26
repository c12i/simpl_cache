use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct TtlCache<K, V> {
    map: Arc<Mutex<HashMap<K, (V, Instant)>>>,
    ttl: Duration,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + std::hash::Hash + Send + 'static,
    V: Clone + Send + 'static,
{
    pub fn new(ttl: Duration) -> Self {
        let map = Arc::new(Mutex::new(HashMap::new()));
        let cache = TtlCache {
            map: map.clone(),
            ttl,
        };
        std::thread::spawn(move || loop {
            map.lock()
                .unwrap()
                .retain(|_, (_, instant)| instant.elapsed() < cache.ttl);
            std::thread::sleep(Duration::from_secs(1));
        });
        cache
    }

    pub fn insert(&self, key: K, value: V) {
        let now = Instant::now();
        self.map.lock().unwrap().insert(key, (value, now));
    }

    pub fn get(&self, key: K) -> Option<V> {
        self.map
            .lock()
            .unwrap()
            .get(&key)
            .map(|(value, _)| value.to_owned())
    }

    pub fn remove(&self, key: K) -> Option<V> {
        self.map
            .lock()
            .unwrap()
            .remove(&key)
            .map(|(value, _)| value)
    }

    pub fn clear(&self) {
        self.map.lock().unwrap().clear();
    }

    pub fn is_empty(&self) -> bool {
        self.map.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_insert() {
        let cache = TtlCache::new(Duration::from_secs(5));
        cache.insert("foo", "test");
        assert_eq!(cache.get("foo"), Some("test"));
    }

    #[test]
    fn test_cache_get_expired() {
        let cache = TtlCache::new(Duration::from_secs(1));
        cache.insert("foo", "test");
        std::thread::sleep(Duration::from_secs(2));
        assert_eq!(cache.get("foo"), None);
    }

    #[test]
    fn test_cache_remove() {
        let cache = TtlCache::new(Duration::from_secs(5));
        cache.insert("foo", "test");
        assert_eq!(cache.get("foo"), Some("test"));
        cache.remove("foo");
        assert_eq!(cache.get("foo"), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = TtlCache::new(Duration::from_secs(5));
        assert!(cache.is_empty());
        cache.insert("foo", "test");
        assert_eq!(cache.get("foo"), Some("test"));
        cache.clear();
        assert!(cache.is_empty());
    }
}
