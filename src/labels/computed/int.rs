//! [`IntLabeling`]: a random integer for every element.

use std::ops::Range;

use super::super::labeling::name_error;
use super::super::{Error, Errors, Kind, Label, Labeling};
use crate::Context;
use crate::random::{self, Key};

/// A computed labeling that gives every element a pseudo-random integer in a half-open range
/// `a..b`. A label is a function of the master seed (in the context), the stream and the element
/// only.
///
/// The constructor does not check its parameters; [`validate`](Labeling::validate) reports every
/// invalid one: [`Error::InvalidName`] if the name is not an identifier, and [`Error::EmptyRange`]
/// if the range has no numbers (`start >= end`). Reading an invalid labeling never panics, but the
/// labels are unspecified.
#[derive(Debug, Clone, PartialEq)]
pub struct IntLabeling {
    name: String,
    start: i64,
    end: i64,
    stream: u64,
}

impl IntLabeling {
    /// The labeling called `name` with labels in `range`, on the stream derived from the name.
    pub fn new(name: impl Into<String>, range: Range<i64>) -> IntLabeling {
        let name = name.into();
        let stream = random::name_stream(&name);
        IntLabeling {
            name,
            start: range.start,
            end: range.end,
            stream,
        }
    }

    /// The same labeling on the stream number `stream`, in place of the default derived from the
    /// name ([`name_stream`](crate::random::name_stream)).
    pub fn with_stream(self, stream: u64) -> IntLabeling {
        IntLabeling { stream, ..self }
    }
}

impl<E: Key> Labeling<E> for IntLabeling {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &Kind {
        &Kind::Int
    }

    fn label(&self, ctx: &Context, element: E) -> Option<Label> {
        let seed = random::stream_seed(ctx.seed, self.stream);
        // At most 2^64 - 1, so it fits a u64 for every valid range; an empty one reads as 1.
        let span = u64::try_from(i128::from(self.end) - i128::from(self.start))
            .unwrap_or(1)
            .max(1);
        let offset = random::bounded(element.fold(seed), span);
        Some(Label::Int(self.start.wrapping_add_unsigned(offset)))
    }

    fn validate(&self) -> Result<(), Errors> {
        let mut problems: Vec<Error> = name_error(&self.name).into_iter().collect();
        if self.start >= self.end {
            problems.push(Error::EmptyRange);
        }
        Errors::from_list(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{context_at, label_problems};

    /// Stream 0 under the master seed 5, which is how the reference vectors were made.
    fn ints(range: Range<i64>) -> IntLabeling {
        IntLabeling::new("w", range).with_stream(0)
    }

    fn int_under(l: &IntLabeling, seed: u64, node: u64) -> i64 {
        match Labeling::<u64>::label(l, &context_at(seed), node) {
            Some(Label::Int(i)) => i,
            other => panic!("not an integer: {other:?}"),
        }
    }

    fn int_of(l: &IntLabeling, node: u64) -> i64 {
        int_under(l, 5, node)
    }

    #[test]
    fn labels_follow_the_reference_vector() {
        // Computed by an independent implementation of `mix` and `bounded`.
        let l = ints(1..10);
        let got: Vec<_> = (0..8).map(|v| int_of(&l, v)).collect();
        assert_eq!(got, [1, 5, 4, 4, 6, 8, 8, 6]);
        let l = ints(-5..5);
        let got: Vec<_> = (0..8).map(|v| int_of(&l, v)).collect();
        assert_eq!(got, [-5, 4, 3, -3, -4, 1, -2, -2]);
    }

    #[test]
    fn the_upper_end_is_not_a_label_and_everything_else_is() {
        let l = ints(1..6);
        let seen: std::collections::BTreeSet<i64> = (0..500).map(|v| int_of(&l, v)).collect();
        assert_eq!(seen, (1..6).collect());
    }

    #[test]
    fn a_range_of_one_number_gives_that_number() {
        let l = ints(7..8);
        assert!((0..50).all(|v| int_of(&l, v) == 7));
    }

    #[test]
    fn the_whole_i64_range_works() {
        let l = ints(i64::MIN..i64::MAX);
        let values: Vec<_> = (0..100).map(|v| int_of(&l, v)).collect();
        assert!(values.iter().any(|&x| x < 0) && values.iter().any(|&x| x > 0));
        assert!(values.iter().all(|&x| x != i64::MAX));
    }

    #[test]
    fn counts_are_close_to_uniform() {
        let l = ints(0..4);
        let mut counts = [0u32; 4];
        for v in 0..20_000 {
            counts[int_of(&l, v) as usize] += 1;
        }
        for c in counts {
            assert!((4_600..5_400).contains(&c), "{counts:?}");
        }
    }

    #[test]
    fn the_master_seed_of_the_context_decides_the_labels() {
        let l = IntLabeling::new("w", 0..1000);
        let of = |seed| (0..50).map(|v| int_under(&l, seed, v)).collect::<Vec<_>>();
        assert_eq!(of(1), of(1));
        assert_ne!(of(1), of(2));
    }

    #[test]
    fn the_stream_decides_the_labels_and_the_default_comes_from_the_name() {
        let by_name = IntLabeling::new("w", 0..1000);
        let other_name = IntLabeling::new("v", 0..1000);
        let of = |l: &IntLabeling| (0..50).map(|v| int_under(l, 3, v)).collect::<Vec<_>>();
        // The default stream is the hash of the name, so it can be set to the same thing.
        let same = IntLabeling::new("x", 0..1000).with_stream(random::name_stream("w"));
        assert_eq!(of(&by_name), of(&same));
        // Another name or another stream is independent of it.
        assert_ne!(of(&by_name), of(&other_name));
        assert_ne!(of(&by_name), of(&by_name.clone().with_stream(0)));
        // Stream 0 under a seed is the same as the seed itself, which is what the vectors use.
        assert_eq!(random::stream_seed(5, 0), 5);
    }

    #[test]
    fn labels_nodes_and_pairs_alike() {
        let l = ints(0..1000);
        let ctx = context_at(5);
        assert_eq!(
            Labeling::<u64>::label(&l, &ctx, 3),
            Labeling::<u64>::label(&l, &ctx, 3)
        );
        let ab = l.label(&ctx, (1u64, 2u64));
        let ba = l.label(&ctx, (2u64, 1u64));
        assert!(ab.is_some() && ba.is_some());
        assert_ne!(
            (0..50u64)
                .map(|i| l.label(&ctx, (i, 0u64)))
                .collect::<Vec<_>>(),
            (0..50u64)
                .map(|i| l.label(&ctx, (0u64, i)))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn reads_do_not_depend_on_the_order() {
        let l = ints(0..100);
        let forward: Vec<_> = (0..20).map(|v| int_of(&l, v)).collect();
        let backward: Vec<_> = (0..20).rev().map(|v| int_of(&l, v)).collect();
        assert_eq!(forward, backward.into_iter().rev().collect::<Vec<_>>());
    }

    #[test]
    fn check_reports_every_problem_in_order() {
        let check = |name: &str, range| label_problems(&IntLabeling::new(name, range));
        assert_eq!(check("w", 3..4), vec![]);
        assert_eq!(check("w", 3..3), vec![Error::EmptyRange]);
        let reversed = Range { start: 4, end: 3 };
        assert_eq!(check("w", reversed), vec![Error::EmptyRange]);
        assert_eq!(
            check("2w", 0..3),
            vec![Error::InvalidName("2w".to_string())]
        );
        // Both problems, the name first.
        assert_eq!(
            check("2w", 3..3),
            vec![Error::InvalidName("2w".to_string()), Error::EmptyRange]
        );
    }

    #[test]
    fn an_invalid_labeling_can_still_be_read() {
        let l = IntLabeling::new("w", 3..3);
        assert!((0..10).all(|v| Labeling::<u64>::label(&l, &context_at(1), v).is_some()));
    }
}
