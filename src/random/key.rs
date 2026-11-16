//! [`Key`]: what identifies a counter-based draw.

use super::hash::mix;

/// A key that identifies a counter-based draw: something that can fold itself into a running hash.
/// Implement it by mixing the integers that identify the key with [`mix`](super::mix), in a fixed
/// order; the standard `Hash` is not used, since its output is not stable across platforms and
/// versions.
pub trait Key {
    /// Folds this key into `state` and returns the new state.
    fn fold(&self, state: u64) -> u64;
}

impl Key for u64 {
    fn fold(&self, state: u64) -> u64 {
        mix(state, *self)
    }
}

impl Key for (u64, u64) {
    fn fold(&self, state: u64) -> u64 {
        mix(mix(state, self.0), self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_folds_by_one_mix() {
        assert_eq!(5u64.fold(9), 0xd59b_a69e_d278_d3f3);
        assert_eq!(5u64.fold(9), mix(9, 5));
    }

    #[test]
    fn a_pair_folds_in_order() {
        assert_eq!((3u64, 4u64).fold(9), 0xee43_fb1c_b276_c6f3);
        assert_eq!((4u64, 3u64).fold(9), 0x4bc7_4892_53ff_2b58);
    }
}
