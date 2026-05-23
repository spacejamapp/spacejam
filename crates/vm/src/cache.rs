//! Lock-free read-mostly cache keyed by program hash.

use arc_swap::ArcSwap;
use score::OpaqueHash;
use std::{collections::HashMap, sync::Arc};

/// Lock-free hash-keyed cache holding `Arc<V>`.
pub struct Cache<V> {
    inner: ArcSwap<HashMap<OpaqueHash, Arc<V>>>,
}

impl<V> Cache<V> {
    /// Look up by hash; returns a cheap `Arc` clone on hit.
    pub fn get(&self, hash: &OpaqueHash) -> Option<Arc<V>> {
        self.inner.load().get(hash).cloned()
    }

    /// Insert or replace.
    pub fn put(&self, hash: OpaqueHash, value: Arc<V>) {
        self.inner.rcu(|prev| {
            let mut next = (**prev).clone();
            next.insert(hash, value.clone());
            next
        });
    }
}

impl<V> Default for Cache<V> {
    fn default() -> Self {
        Self {
            inner: ArcSwap::from_pointee(HashMap::new()),
        }
    }
}
