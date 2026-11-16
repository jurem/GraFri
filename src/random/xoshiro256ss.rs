//! xoshiro256** PRNG.

use super::{Prng, SplitMix64};

/// xoshiro256** 1.0 (Blackman & Vigna, 2018). The state words are the first four outputs of
/// [`SplitMix64`] started from the seed.
#[derive(Debug, Clone)]
pub struct Xoshiro256StarStar {
    s: [u64; 4],
}

impl Xoshiro256StarStar {
    /// The generator for a `u64` seed; every seed is valid.
    pub fn from_seed(seed: u64) -> Xoshiro256StarStar {
        let mut sm = SplitMix64::new(seed);
        Xoshiro256StarStar {
            s: [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()],
        }
    }
}

impl Prng for Xoshiro256StarStar {
    fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_stream_from_raw_state() {
        // Reference algorithm started from the state {1, 2, 3, 4}.
        let mut rng = Xoshiro256StarStar { s: [1, 2, 3, 4] };
        let got: Vec<u64> = (0..4).map(|_| rng.next_u64()).collect();
        assert_eq!(got, [11520, 0, 1_509_978_240, 1_215_971_899_390_074_240]);
    }

    #[test]
    fn pinned_seeded_stream() {
        // Reference C code (prng.di.unimi.it) seeded with splitmix64(42).
        // If this changes, every generated graph changes.
        let mut rng = Xoshiro256StarStar::from_seed(42);
        let got: Vec<u64> = (0..3).map(|_| rng.next_u64()).collect();
        assert_eq!(
            got,
            [
                0x1578_0B2E_0C2E_C716,
                0x6104_D986_6D11_3A7E,
                0xAE17_5332_39E4_99A1
            ]
        );
    }
}
