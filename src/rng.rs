//! Seeded RNG (xorshift32) — the sim's only source of variation.
//!
//! Determinism contract: same seed + same input tape = same draw lists.
//! A zero seed is remapped to a fixed nonzero constant so the generator
//! can never park at its absorbing state.

/// Xorshift32 generator. Period 2^32 - 1 for any nonzero state.
pub struct Rng {
    state: u32,
}

impl Rng {
    /// Create a generator from any seed; 0 is remapped.
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9 } else { seed },
        }
    }

    /// Next raw u32.
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Next value in 0..bound (bound must be positive; a non-positive
    /// bound yields 0 rather than a panic).
    pub fn below(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        (self.next_u32() % (bound as u32)) as i32
    }

    /// Symmetric jitter in -range..=range (milli-units by convention).
    pub fn jitter(&mut self, range: i32) -> i32 {
        if range <= 0 {
            return 0;
        }
        self.below(2 * range + 1) - range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn zero_seed_does_not_park() {
        let mut rng = Rng::new(0);
        let first = rng.next_u32();
        assert_ne!(first, 0);
        assert_ne!(rng.next_u32(), first);
    }

    #[test]
    fn below_respects_bound() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let v = rng.below(10);
            assert!((0..10).contains(&v));
        }
        assert_eq!(rng.below(0), 0);
        assert_eq!(rng.below(-3), 0);
    }
}
