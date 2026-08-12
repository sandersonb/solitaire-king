//! A tiny, self-contained deterministic PRNG.
//!
//! We deliberately implement [SplitMix64] in-crate rather than depend on an
//! external RNG. This keeps deals reproducible and portable: the algorithm is
//! fixed and auditable here, so a given seed yields the same sequence on every
//! platform and every version of this crate. Changing this algorithm is a
//! breaking change to deal reproducibility.
//!
//! [SplitMix64]: https://prng.di.unimi.it/splitmix64.c

/// A deterministic SplitMix64 pseudo-random number generator.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Create a generator seeded by `seed`. Every seed value is valid.
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    /// Return the next 64-bit pseudo-random value and advance the state.
    pub fn next_u64(&mut self) -> u64 {
        // SplitMix64: advance by the golden-ratio odd constant, then mix.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Return a uniformly distributed value in `0..bound` without modulo bias.
    ///
    /// Uses Lemire's multiply-and-shift rejection method. `bound` must be > 0.
    pub fn next_below(&mut self, bound: u64) -> u64 {
        debug_assert!(bound > 0, "bound must be positive");
        // Lemire's method: reject the low zone that would cause bias.
        let mut x = self.next_u64();
        let mut m = (x as u128).wrapping_mul(bound as u128);
        let mut low = m as u64;
        if low < bound {
            // threshold = (2^64 - bound) % bound, computed as (-bound) % bound.
            let threshold = bound.wrapping_neg() % bound;
            while low < threshold {
                x = self.next_u64();
                m = (x as u128).wrapping_mul(bound as u128);
                low = m as u64;
            }
        }
        (m >> 64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        // Extremely unlikely to collide on the very first draw.
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn next_below_in_range() {
        let mut r = SplitMix64::new(7);
        for _ in 0..10_000 {
            let v = r.next_below(52);
            assert!(v < 52);
        }
    }
}
