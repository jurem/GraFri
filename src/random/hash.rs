//! The counter-based primitives: random bits that are a pure function of a seed and a key, not a
//! draw from a sequence. The bits for a key are `key.fold(seed)`, a chain of [`mix`] steps. Part of
//! the reproducibility contract: changing any of them changes every computed label.

const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// The SplitMix64 output function (without its increment).
fn finalize(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Mixes `word` into `state`: `finalize(finalize(state + GAMMA) + word)`, all wrapping. Two rounds,
/// so that `state` and `word` play different roles.
pub fn mix(state: u64, word: u64) -> u64 {
    finalize(finalize(state.wrapping_add(GAMMA)).wrapping_add(word))
}

/// A float in `[0, 1)` from 53 of the bits, as [`Prng::next_f64`](super::Prng::next_f64)
/// makes it.
pub fn unit(bits: u64) -> f64 {
    (bits >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// An unbiased number in `0..n` from `bits`: a word in the first `2^64 mod n` values is rejected,
/// and the next word is `mix(bits, k)` for `k = 1, 2, ...`. `n` must be above zero (for zero the
/// result is zero).
pub fn bounded(bits: u64, n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let reject_below = n.wrapping_neg() % n;
    let mut word = bits;
    let mut k = 0;
    while word < reject_below {
        k += 1;
        word = mix(bits, k);
    }
    word % n
}

/// The default random stream of a name: `mix` folded over the length of its UTF-8 bytes, then over
/// each byte, from state 0. Part of the reproducibility contract.
pub fn name_stream(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut state = mix(0, bytes.len() as u64);
    for &byte in bytes {
        state = mix(state, u64::from(byte));
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_stream_follows_the_reference_vector() {
        // From an independent Python implementation of the definition; its `mix` reproduces `mix(9,
        // 5) = 0xd59ba69ed278d3f3`.
        assert_eq!(name_stream(""), 0x4821_8226_ff3c_d4bf);
        assert_eq!(name_stream("a"), 0x7888_8897_9fb8_4c89);
        assert_eq!(name_stream("color"), 0x121c_d6ef_9d7f_d67e);
        assert_eq!(name_stream("weight"), 0xc4c7_c7c6_9fa1_54e6);
        assert_eq!(name_stream("w"), 0x2165_d08b_6ad4_d79d);
        // Bytes, not characters: a name with a non-ASCII letter.
        assert_eq!(name_stream("žaba"), 0x1e4f_aa3d_ead1_a6fd);
    }

    #[test]
    fn different_names_give_different_streams() {
        let names = ["a", "b", "aa", "ab", "ba", "color", "colour", ""];
        let streams: std::collections::BTreeSet<u64> =
            names.iter().map(|n| name_stream(n)).collect();
        assert_eq!(streams.len(), names.len());
    }

    #[test]
    fn mix_reference_vectors() {
        // Computed by an independent implementation of the definition above.
        assert_eq!(mix(0, 0), 0x4821_8226_ff3c_d4bf);
        assert_eq!(mix(1, 2), 0x081a_5c13_7785_6b73);
        assert_eq!(mix(2, 1), 0xf282_6f98_653e_9e57);
        assert_eq!(mix(u64::MAX, u64::MAX), 0xe8ba_9f99_ca93_3538);
        assert_eq!(mix(42, 7), 0xb1ea_561e_1a81_9115);
    }

    #[test]
    fn state_and_word_are_not_interchangeable() {
        assert_ne!(mix(1, 2), mix(2, 1));
        assert_ne!(mix(5, 0), mix(0, 5));
    }

    #[test]
    fn unit_is_in_zero_one() {
        assert_eq!(unit(0), 0.0);
        assert_eq!(unit(u64::MAX), 1.0 - 2f64.powi(-53));
        for i in 0..1000 {
            let u = unit(mix(3, i));
            assert!((0.0..1.0).contains(&u));
        }
    }

    #[test]
    fn bounded_stays_in_range() {
        assert_eq!(bounded(123, 0), 0);
        assert_eq!(bounded(123, 1), 0);
        for n in [2u64, 3, 10, 1000, u64::MAX] {
            for i in 0..200 {
                assert!(bounded(mix(7, i), n) < n, "n={n}");
            }
        }
        assert_eq!(bounded(mix(1, 1), 10), 9);
    }

    #[test]
    fn bounded_rejects_words_that_would_bias_the_result() {
        // n = 2^63 + 1 rejects every word below 2^63 - 1, and bits 0 is one of them,
        // so the result is taken from the next word, `mix(0, 1)`.
        let n = (1u64 << 63) + 1;
        assert_eq!(n.wrapping_neg() % n, (1u64 << 63) - 1);
        assert!(mix(0, 1) >= (1u64 << 63) - 1);
        assert_eq!(bounded(0, n), 7_675_656_394_906_785_482);
        assert_eq!(bounded(0, n), mix(0, 1) % n);
    }

    #[test]
    fn bounded_is_close_to_uniform() {
        let mut counts = [0u32; 3];
        for i in 0..30_000 {
            counts[bounded(mix(11, i), 3) as usize] += 1;
        }
        for c in counts {
            assert!((9_500..10_500).contains(&c), "{counts:?}");
        }
    }
}
