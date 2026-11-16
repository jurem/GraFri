//! SplitMix64.

use super::Prng;

/// SplitMix64 (Steele, Lea & Flood, 2014), in Vigna's reference form. It seeds
/// [`Xoshiro256StarStar`](super::Xoshiro256StarStar).
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

/// SplitMix64's increment, `2^64 / phi`.
pub(crate) const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

impl SplitMix64 {
    /// A generator starting from `seed`; any value is valid.
    pub fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }
}

impl Prng for SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_vector_seed_0() {
        // From the reference C code (prng.di.unimi.it/splitmix64.c).
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(rng.next_u64(), 0x06C4_5D18_8009_454F);
    }
}
