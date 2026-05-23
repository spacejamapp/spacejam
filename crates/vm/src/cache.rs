//! Lock-free read-mostly cache keyed by program hash, bounded by FIFO eviction.

use arc_swap::ArcSwap;
use score::OpaqueHash;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

/// Default cache capacity when neither `with_capacity` nor `SPACEJAM_CACHE_CAPACITY` is set.
const DEFAULT_CAPACITY: usize = 32;

/// Lock-free hash-keyed cache holding `Arc<V>` with FIFO eviction at capacity.
pub struct Cache<V> {
    inner: ArcSwap<Inner<V>>,
}

impl<V> Cache<V> {
    /// Construct a cache with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: ArcSwap::from_pointee(Inner::new(capacity)),
        }
    }

    /// Look up by hash; returns a cheap `Arc` clone on hit.
    pub fn get(&self, hash: &OpaqueHash) -> Option<Arc<V>> {
        self.inner.load().map.get(hash).cloned()
    }

    /// Insert or replace.
    pub fn put(&self, hash: OpaqueHash, value: Arc<V>) {
        self.inner.rcu(|prev| {
            let mut next = prev.clone_shape();
            let exists = next.map.contains_key(&hash);
            if !exists && next.map.len() >= next.capacity {
                if let Some(evict) = next.order.pop_front() {
                    next.map.remove(&evict);
                }
            }
            next.map.insert(hash, value.clone());
            if !exists {
                next.order.push_back(hash);
            }
            next
        });
    }
}

impl<V> Default for Cache<V> {
    fn default() -> Self {
        Self::with_capacity(default_capacity())
    }
}

struct Inner<V> {
    map: HashMap<OpaqueHash, Arc<V>>,
    order: VecDeque<OpaqueHash>,
    capacity: usize,
}

impl<V> Inner<V> {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn clone_shape(&self) -> Self {
        Self {
            map: self.map.clone(),
            order: self.order.clone(),
            capacity: self.capacity,
        }
    }
}

fn default_capacity() -> usize {
    std::env::var("SPACEJAM_CACHE_CAPACITY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_CAPACITY)
}
