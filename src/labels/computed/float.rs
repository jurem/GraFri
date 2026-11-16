//! [`FloatLabeling`]: a random float for every element.

use std::ops::Range;

use super::super::labeling::name_error;
use super::super::{Error, Errors, Kind, Label, Labeling};
use crate::Context;
use crate::random::{self, Key};

/// A computed labeling that gives every element a pseudo-random float in a half-open range `a..b`.
/// A label is a function of the master seed (in the context), the stream and the element only,
/// `a + (b - a) * u` for a uniform `u` in `[0, 1)`, so it is the same on every platform.
///
/// The constructor does not check its parameters; [`validate`](Labeling::validate) reports every
/// invalid one: [`Error::InvalidName`] if the name is not an identifier, and
/// [`Error::InvalidFloatRange`] if a bound or the width is not finite, or `start >= end`. Reading
/// an invalid labeling never panics, but the labels are unspecified.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatLabeling {
    name: String,
    start: f64,
    end: f64,
    stream: u64,
}

impl FloatLabeling {
    /// The labeling called `name` with labels in `range`, on the stream derived from the name.
    pub fn new(name: impl Into<String>, range: Range<f64>) -> FloatLabeling {
        let name = name.into();
        let stream = random::name_stream(&name);
        FloatLabeling {
            name,
            start: range.start,
            end: range.end,
            stream,
        }
    }

    /// The same labeling on the stream number `stream`, in place of the default derived from the
    /// name ([`name_stream`](crate::random::name_stream)).
    pub fn with_stream(self, stream: u64) -> FloatLabeling {
        FloatLabeling { stream, ..self }
    }
}

impl<E: Key> Labeling<E> for FloatLabeling {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &Kind {
        &Kind::Float
    }

    fn label(&self, ctx: &Context, element: E) -> Option<Label> {
        let seed = random::stream_seed(ctx.seed, self.stream);
        let x = self.start + (self.end - self.start) * random::unit(element.fold(seed));
        // Rounding can reach `end`; the label must stay below it.
        Some(Label::Float(if x < self.end {
            x
        } else {
            self.end.next_down()
        }))
    }

    fn validate(&self) -> Result<(), Errors> {
        let mut problems: Vec<Error> = name_error(&self.name).into_iter().collect();
        let (start, end) = (self.start, self.end);
        if !(start.is_finite() && end.is_finite() && start < end && (end - start).is_finite()) {
            problems.push(Error::InvalidFloatRange);
        }
        Errors::from_list(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{context_at, label_problems};

    /// Stream 0 under the master seed 5, which is how the reference vectors were made.
    fn floats(range: Range<f64>) -> FloatLabeling {
        FloatLabeling::new("x", range).with_stream(0)
    }

    fn float_under(l: &FloatLabeling, seed: u64, node: u64) -> f64 {
        match Labeling::<u64>::label(l, &context_at(seed), node) {
            Some(Label::Float(x)) => x,
            other => panic!("not a float: {other:?}"),
        }
    }

    fn float_of(l: &FloatLabeling, node: u64) -> f64 {
        float_under(l, 5, node)
    }

    #[test]
    fn labels_follow_the_reference_vector() {
        // Computed by an independent implementation of `mix` and `unit`.
        let l = floats(-1.5..2.5);
        let got: Vec<_> = (0..6).map(|v| float_of(&l, v)).collect();
        assert_eq!(
            got,
            [
                1.1894719707367947,
                0.5150898596495588,
                0.835968955463223,
                1.9349279158476373,
                0.32424300952022866,
                1.8260436320170959
            ]
        );
        assert_eq!(
            l.label(&context_at(5), (3u64, 4u64)),
            Some(Label::Float(2.043052490750575))
        );
    }

    #[test]
    fn labels_stay_in_the_half_open_range() {
        let l = floats(-1.5..2.5);
        assert!((0..5000).all(|v| (-1.5..2.5).contains(&float_of(&l, v))));
    }

    #[test]
    fn the_upper_end_is_never_reached_even_by_adjacent_floats() {
        let a = 1.0f64;
        let b = a.next_up();
        let l = floats(a..b);
        assert!((0..2000).all(|v| float_of(&l, v) == a));
    }

    #[test]
    fn the_mean_is_close_to_the_middle() {
        let l = floats(0.0..10.0);
        let mean = (0..20_000).map(|v| float_of(&l, v)).sum::<f64>() / 20_000.0;
        assert!((4.9..5.1).contains(&mean), "{mean}");
    }

    #[test]
    fn the_master_seed_and_the_stream_decide_the_labels() {
        let l = FloatLabeling::new("x", 0.0..1.0);
        let of =
            |l: &FloatLabeling, seed| (0..20).map(|v| float_under(l, seed, v)).collect::<Vec<_>>();
        assert_eq!(of(&l, 1), of(&l, 1));
        assert_ne!(of(&l, 1), of(&l, 2));
        assert_ne!(of(&l, 1), of(&l.clone().with_stream(0), 1));
        let other_name = FloatLabeling::new("y", 0.0..1.0);
        assert_ne!(of(&l, 1), of(&other_name, 1));
    }

    #[test]
    fn check_reports_every_problem_in_order() {
        let reversed = Range {
            start: 1.0,
            end: 0.0,
        };
        for range in [
            1.0..1.0,
            reversed,
            f64::NAN..1.0,
            0.0..f64::NAN,
            f64::NEG_INFINITY..0.0,
            0.0..f64::INFINITY,
            -f64::MAX..f64::MAX,
        ] {
            assert_eq!(
                label_problems(&FloatLabeling::new("x", range.clone())),
                vec![Error::InvalidFloatRange],
                "{range:?}"
            );
        }
        assert_eq!(
            label_problems(&FloatLabeling::new("2x", 0.0..1.0)),
            vec![Error::InvalidName("2x".to_string())]
        );
        // Both problems, the name first.
        assert_eq!(
            label_problems(&FloatLabeling::new("2x", 1.0..1.0)),
            vec![
                Error::InvalidName("2x".to_string()),
                Error::InvalidFloatRange
            ]
        );
        assert_eq!(label_problems(&floats(0.0..1.0)), vec![]);
    }

    #[test]
    fn an_invalid_labeling_can_still_be_read() {
        let l = FloatLabeling::new("x", f64::NAN..1.0);
        let label = Labeling::<u64>::label(&l, &context_at(1), 3).unwrap();
        assert!(Labeling::<u64>::kind(&l).accepts(&label));
    }
}
