//! [`ChoiceLabeling`]: a random variant of an enumeration for every element.

use super::super::labeling::name_error;
use super::super::{Error, Errors, Kind, Label, Labeling, Variants};
use crate::Context;
use crate::random::{self, Key};

/// A computed labeling that gives every element one of the variants of an enumeration, each with
/// the same probability and independently. A label is a function of the master seed (in the
/// context), the stream and the element only.
///
/// The constructor does not check its parameters; [`validate`](Labeling::validate) reports every
/// invalid one: [`Error::InvalidName`] if the name is not an identifier, and the error of
/// [`Variants::validate`] for bad variant names. Reading an invalid labeling never panics, but the
/// labels are unspecified.
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceLabeling {
    name: String,
    variants: Variants,
    stream: u64,
}

impl ChoiceLabeling {
    /// The labeling called `name` that chooses among the variants named
    /// `variants`, on the stream derived from the name.
    pub fn new<S: Into<String>>(
        name: impl Into<String>,
        variants: impl IntoIterator<Item = S>,
    ) -> ChoiceLabeling {
        let name = name.into();
        let stream = random::name_stream(&name);
        ChoiceLabeling {
            name,
            variants: Variants::new(variants),
            stream,
        }
    }

    /// The same labeling on the stream number `stream`, in place of the default derived from the
    /// name ([`name_stream`](crate::random::name_stream)).
    pub fn with_stream(self, stream: u64) -> ChoiceLabeling {
        ChoiceLabeling { stream, ..self }
    }
}

impl<E: Key> Labeling<E> for ChoiceLabeling {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &Kind {
        &Kind::Enum
    }

    fn variants(&self) -> Option<&Variants> {
        Some(&self.variants)
    }

    fn label(&self, ctx: &Context, element: E) -> Option<Label> {
        let seed = random::stream_seed(ctx.seed, self.stream);
        // The number of variants fits a u32 (checked by `Variants`), and so does the result.
        let count = self.variants.names().len() as u64;
        let index = random::bounded(element.fold(seed), count);
        Some(Label::Enum(index as u32))
    }

    fn validate(&self) -> Result<(), Errors> {
        let problems: Vec<Error> = name_error(&self.name)
            .into_iter()
            .chain(self.variants.validate().err())
            .collect();
        Errors::from_list(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::Error;
    use super::*;
    use crate::testing::{context_at, label_problems};

    /// Stream 0 under the master seed 5, which is how the reference vectors were made.
    fn colors() -> ChoiceLabeling {
        ChoiceLabeling::new("color", ["red", "green", "blue"]).with_stream(0)
    }

    fn index_under(l: &ChoiceLabeling, seed: u64, node: u64) -> u32 {
        match Labeling::<u64>::label(l, &context_at(seed), node) {
            Some(Label::Enum(i)) => i,
            other => panic!("not an enumeration label: {other:?}"),
        }
    }

    fn index_of(l: &ChoiceLabeling, node: u64) -> u32 {
        index_under(l, 5, node)
    }

    #[test]
    fn labels_follow_the_reference_vector() {
        // Computed by an independent implementation of `mix` and `bounded`.
        let l = colors();
        let got: Vec<_> = (0..8).map(|v| index_of(&l, v)).collect();
        assert_eq!(got, [0, 1, 0, 0, 2, 1, 1, 2]);
        assert_eq!(l.label(&context_at(5), (1u64, 2u64)), Some(Label::Enum(0)));
    }

    #[test]
    fn every_variant_is_used_and_the_counts_are_close() {
        let l = colors();
        let mut counts = [0u32; 3];
        for v in 0..30_000 {
            counts[index_of(&l, v) as usize] += 1;
        }
        for c in counts {
            assert!((9_500..10_500).contains(&c), "{counts:?}");
        }
    }

    #[test]
    fn a_single_variant_is_always_chosen() {
        let l = ChoiceLabeling::new("only", ["x"]);
        assert!((0..100).all(|v| index_of(&l, v) == 0));
    }

    #[test]
    fn the_master_seed_and_the_stream_decide_the_labels() {
        let l = ChoiceLabeling::new("c", ["a", "b", "c", "d"]);
        let of =
            |l: &ChoiceLabeling, seed| (0..64).map(|v| index_under(l, seed, v)).collect::<Vec<_>>();
        assert_eq!(of(&l, 1), of(&l, 1));
        assert_ne!(of(&l, 1), of(&l, 2));
        assert_ne!(of(&l, 1), of(&l.clone().with_stream(0), 1));
        let other_name = ChoiceLabeling::new("d", ["a", "b", "c", "d"]);
        assert_ne!(of(&l, 1), of(&other_name, 1));
    }

    #[test]
    fn check_reports_every_problem_in_order() {
        let check = |name: &str, variants: &[&str]| {
            label_problems(&ChoiceLabeling::new(name, variants.iter().copied()))
        };
        assert_eq!(check("c", &["a", "b"]), vec![]);
        assert_eq!(check("c", &[]), vec![Error::NoVariants]);
        assert_eq!(check("c", &["a", ""]), vec![Error::EmptyVariant]);
        assert_eq!(
            check("c", &["a", "a"]),
            vec![Error::RepeatedVariant("a".to_string())]
        );
        assert_eq!(
            check("2c", &["a"]),
            vec![Error::InvalidName("2c".to_string())]
        );
        // Both problems, the name first.
        assert_eq!(
            check("2c", &[]),
            vec![Error::InvalidName("2c".to_string()), Error::NoVariants]
        );
    }

    #[test]
    fn labels_are_formatted_as_the_variant_names() {
        let l = colors();
        assert_eq!(Labeling::<u64>::variants(&l).unwrap().names().len(), 3);
        assert_eq!(
            Labeling::<u64>::format(&l, &Label::Enum(2)).unwrap(),
            "blue"
        );
        assert_eq!(Labeling::<u64>::format(&l, &Label::Enum(3)), None);
    }

    #[test]
    fn an_invalid_labeling_can_still_be_read() {
        let l = ChoiceLabeling::new("c", Vec::<String>::new());
        assert!(Labeling::<u64>::label(&l, &context_at(1), 3).is_some());
    }
}
