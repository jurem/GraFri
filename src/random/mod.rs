//! Randomness. All of it in grafri comes from here, so that a seed determines the output on every
//! platform.
//!
//! The sequential generators implement [`Prng`] (SplitMix64 and xoshiro256**): each draw changes a
//! state. The counter-based primitives ([`mix`], [`unit()`], [`bounded`], with [`Key`]) have no
//! state: the bits are a pure function of a seed and a key.

mod hash;
mod key;
mod splitmix64;
mod xoshiro256ss;

pub use hash::{bounded, mix, name_stream, unit};
pub use key::Key;
pub use splitmix64::SplitMix64;
pub use xoshiro256ss::Xoshiro256StarStar;

/// The seed of the random stream number `stream` under the master seed `seed`:
/// `seed + stream * GAMMA` (wrapping), where `GAMMA` is SplitMix64's increment. Stream 0 is the
/// master seed itself. Part of the reproducibility contract.
pub fn stream_seed(seed: u64, stream: u64) -> u64 {
    seed.wrapping_add(stream.wrapping_mul(splitmix64::GAMMA))
}

/// A source of random bits. Implementations provide only [`next_u64`](Prng::next_u64); the sampling
/// helpers are defined here once.
pub trait Prng {
    /// The next 64 random bits.
    fn next_u64(&mut self) -> u64;

    /// A uniform number in `[0, 1)` with 53 random bits (one `next_u64` draw).
    fn next_f64(&mut self) -> f64 {
        unit(self.next_u64())
    }

    /// `true` with probability `p` (one draw, always made). `p <= 0` is never true and `p >= 1`
    /// always.
    fn bernoulli(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns a fixed sequence, repeating the last value.
    struct Fixed(Vec<u64>);

    impl Prng for Fixed {
        fn next_u64(&mut self) -> u64 {
            if self.0.len() > 1 {
                self.0.remove(0)
            } else {
                self.0[0]
            }
        }
    }

    #[test]
    fn next_f64_uses_the_top_53_bits() {
        assert_eq!(Fixed(vec![0]).next_f64(), 0.0);
        assert_eq!(Fixed(vec![1 << 63]).next_f64(), 0.5);
        assert_eq!(Fixed(vec![u64::MAX]).next_f64(), 1.0 - 2f64.powi(-53));
    }

    #[test]
    fn bernoulli_edge_cases() {
        for bits in [0, 1 << 63, u64::MAX] {
            assert!(!Fixed(vec![bits]).bernoulli(0.0));
            assert!(Fixed(vec![bits]).bernoulli(1.0));
        }
        assert!(Fixed(vec![1 << 62]).bernoulli(0.5));
        assert!(!Fixed(vec![1 << 63]).bernoulli(0.5));
    }

    #[test]
    fn stream_seed_values() {
        assert_eq!(stream_seed(7, 0), 7);
        assert_eq!(stream_seed(0, 1), 0x9E37_79B9_7F4A_7C15);
        assert_eq!(stream_seed(0, 2), 0x3C6E_F372_FE94_F82A);
        // Wraps around.
        assert_eq!(stream_seed(u64::MAX, 1), 0x9E37_79B9_7F4A_7C14);
    }

    #[test]
    fn streams_and_seeds_give_different_generators() {
        let first =
            |seed, stream| Xoshiro256StarStar::from_seed(stream_seed(seed, stream)).next_u64();
        assert_ne!(first(1, 0), first(1, 1));
        assert_ne!(first(1, 1), first(2, 1));
        assert_eq!(first(1, 1), first(1, 1));
    }
}
