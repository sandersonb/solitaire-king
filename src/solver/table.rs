//! A bounded transposition table (closed set of proven-winless positions).
//!
//! Backed by a `HashSet`, so it resolves hash collisions properly and only
//! evicts when it actually reaches its entry cap — at which point it clears
//! (a generational reset) to stay within bound. Because it stores only
//! proven-winless positions, dropping any is sound: a forgotten entry is simply
//! recomputed if reached again.

use std::collections::HashSet;
use std::hash::Hash;

/// A capped closed set of proven-winless position keys. Below the cap it never
/// evicts (unlike a direct-mapped cache, which sheds entries on collision far
/// below capacity); at the cap it clears to make room. Generic over the key
/// type `K` (a 128-bit Zobrist hash by default, or the exact byte encoding).
pub struct ClosedTable<K> {
    set: HashSet<K>,
    capacity: usize,
    peak: usize,
    evictions: u64,
}

impl<K: Hash + Eq> ClosedTable<K> {
    /// Create a table holding at most `capacity` entries (at least one).
    pub fn with_capacity(capacity: usize) -> Self {
        ClosedTable {
            set: HashSet::new(),
            capacity: capacity.max(1),
            peak: 0,
            evictions: 0,
        }
    }

    /// Whether this position is currently recorded as winless.
    pub fn contains(&self, key: &K) -> bool {
        self.set.contains(key)
    }

    /// Record a proven-winless position. At capacity, clear the table first.
    pub fn insert(&mut self, key: K) {
        if self.set.contains(&key) {
            return;
        }
        if self.set.len() >= self.capacity {
            self.evictions += self.set.len() as u64;
            self.set.clear();
        }
        self.set.insert(key);
        self.peak = self.peak.max(self.set.len());
    }

    /// Peak number of entries retained.
    pub fn peak_entries(&self) -> usize {
        self.peak
    }

    /// Number of entries dropped via clear-on-full (0 while under the cap).
    pub fn evictions(&self) -> u64 {
        self.evictions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::encode::{encode, PositionKey};
    use crate::{GameConfig, GameState};

    fn key_for(seed: u64) -> PositionKey {
        encode(&GameState::new_with_seed(seed, GameConfig::default()))
    }

    #[test]
    fn insert_then_contains() {
        let mut t = ClosedTable::with_capacity(1024);
        let k = key_for(1);
        assert!(!t.contains(&k));
        t.insert(k.clone());
        assert!(t.contains(&k));
    }

    #[test]
    fn no_eviction_below_capacity() {
        // Distinct keys under the cap must all be retained (no collision loss).
        let mut t = ClosedTable::with_capacity(1000);
        for s in 0..500 {
            t.insert(key_for(s));
        }
        assert_eq!(t.evictions(), 0);
        assert_eq!(t.peak_entries(), 500);
        assert!(t.contains(&key_for(0)));
        assert!(t.contains(&key_for(499)));
    }

    #[test]
    fn clears_on_full_and_stays_bounded() {
        // Capacity 1 clears on every new key.
        let mut t = ClosedTable::with_capacity(1);
        let a = key_for(1);
        let b = key_for(2);
        assert_ne!(a, b);
        t.insert(a.clone());
        assert!(t.contains(&a));
        t.insert(b.clone()); // at cap -> clear, then insert b
        assert!(t.contains(&b));
        assert!(!t.contains(&a));
        assert_eq!(t.evictions(), 1);
        assert_eq!(t.peak_entries(), 1);
    }

    #[test]
    fn reinsert_is_idempotent() {
        let mut t = ClosedTable::with_capacity(16);
        let k = key_for(3);
        t.insert(k.clone());
        t.insert(k.clone());
        assert_eq!(t.evictions(), 0);
        assert_eq!(t.peak_entries(), 1);
    }
}
