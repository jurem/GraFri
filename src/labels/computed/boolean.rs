//! [`BoolLabeling`]: a random boolean for every element.

use super::super::labeling::name_error;
use super::super::{Error, Errors, Kind, Label, Labeling};
use crate::Context;
use crate::random::{self, Key};

/// A computed labeling that gives every element `true` with probability `p`, independently. A label
/// is a function of the master seed (in the context), the stream and the element only.
///
/// The constructor does not check its parameters; [`validate`](Labeling::validate) reports every
/// invalid one: [`Error::InvalidName`] if the name is not an identifier, and
/// [`Error::InvalidProbability`] if `p` is not in `[0, 1]`. Reading an invalid labeling never
/// panics, but the labels are unspecified.
#[derive(Debug, Clone, PartialEq)]
pub struct BoolLabeling {
    name: String,
    p: f64,
    stream: u64,
}

impl BoolLabeling {
    /// The labeling called `name` where each label is `true` with probability `p`, on the stream
    /// derived from the name.
    pub fn new(name: impl Into<String>, p: f64) -> BoolLabeling {
        let name = name.into();
        let stream = random::name_stream(&name);
        BoolLabeling { name, p, stream }
    }

    /// The same labeling on the stream number `stream`, in place of the default derived from the
    /// name ([`name_stream`](crate::random::name_stream)).
    pub fn with_stream(self, stream: u64) -> BoolLabeling {
        BoolLabeling { stream, ..self }
    }
}

impl<E: Key> Labeling<E> for BoolLabeling {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &Kind {
        &Kind::Bool
    }

    fn label(&self, ctx: &Context, element: E) -> Option<Label> {
        let seed = random::stream_seed(ctx.seed, self.stream);
        Some(Label::Bool(random::unit(element.fold(seed)) < self.p))
    }

    fn validate(&self) -> Result<(), Errors> {
        let mut problems: Vec<Error> = name_error(&self.name).into_iter().collect();
        if !(0.0..=1.0).contains(&self.p) {
            problems.push(Error::InvalidProbability);
        }
        Errors::from_list(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{context_at, label_problems};

    fn bool_under(l: &BoolLabeling, seed: u64, node: u64) -> bool {
        match Labeling::<u64>::label(l, &context_at(seed), node) {
            Some(Label::Bool(b)) => b,
            other => panic!("not a boolean: {other:?}"),
        }
    }

    /// Stream 0, so that the master seed is the seed of the reference vectors.
    fn coin(p: f64) -> BoolLabeling {
        BoolLabeling::new("b", p).with_stream(0)
    }

    #[test]
    fn labels_follow_the_reference_vector() {
        // Computed by an independent implementation of `mix` and `unit`.
        let l = coin(0.5);
        let got: Vec<_> = (0..12).map(|v| bool_under(&l, 4, v)).collect();
        assert_eq!(
            got,
            [
                true, false, false, true, true, true, false, false, true, true, true, true
            ]
        );
    }

    #[test]
    fn the_extremes_are_certain() {
        let never = coin(0.0);
        let always = coin(1.0);
        assert!((0..2000).all(|v| !bool_under(&never, 1, v)));
        assert!((0..2000).all(|v| bool_under(&always, 1, v)));
    }

    #[test]
    fn the_frequency_is_close_to_p() {
        let l = coin(0.3);
        let hits = (0..20_000).filter(|&v| bool_under(&l, 5, v)).count();
        assert!((5_700..6_300).contains(&hits), "{hits}");
    }

    #[test]
    fn the_master_seed_and_the_stream_decide_the_labels() {
        let l = BoolLabeling::new("b", 0.5);
        let of =
            |l: &BoolLabeling, seed| (0..64).map(|v| bool_under(l, seed, v)).collect::<Vec<_>>();
        assert_eq!(of(&l, 1), of(&l, 1));
        assert_ne!(of(&l, 1), of(&l, 2));
        assert_ne!(of(&l, 1), of(&l.clone().with_stream(0), 1));
        let other_name = BoolLabeling::new("c", 0.5);
        assert_ne!(of(&l, 1), of(&other_name, 1));
    }

    #[test]
    fn labels_pairs_too() {
        let l = coin(0.5);
        let count = (0..200u64)
            .filter(|&i| l.label(&context_at(4), (i, i + 1)) == Some(Label::Bool(true)))
            .count();
        assert!((60..140).contains(&count), "{count}");
    }

    #[test]
    fn check_reports_every_problem_in_order() {
        let check = |name: &str, p| label_problems(&BoolLabeling::new(name, p));
        assert_eq!(check("b", 0.0), vec![]);
        assert_eq!(check("b", 1.0), vec![]);
        for p in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
            assert_eq!(check("b", p), vec![Error::InvalidProbability]);
        }
        assert_eq!(check("2b", 0.5), vec![Error::InvalidName("2b".to_string())]);
        // Both problems, the name first.
        assert_eq!(
            check("2b", 2.0),
            vec![
                Error::InvalidName("2b".to_string()),
                Error::InvalidProbability
            ]
        );
    }

    #[test]
    fn an_invalid_labeling_can_still_be_read() {
        let l = BoolLabeling::new("b", f64::NAN);
        assert!((0..10).all(|v| Labeling::<u64>::label(&l, &context_at(1), v).is_some()));
    }
}
