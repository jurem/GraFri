//! [`ConstantLabeling`]: the same label for every element.

use super::super::labeling::{fits, name_error, variants_error};
use super::super::{Error, Errors, Kind, Label, Labeling, Variants};
use crate::Context;

/// A computed labeling that gives every element the same label. It works for
/// any element type, so it serves nodes and edges alike.
///
/// The constructors do not check their parameters; [`validate`](Labeling::validate) reports every
/// invalid one: [`Error::InvalidName`] if the name is not an identifier, the error of
/// [`Variants::validate`] for bad variants, [`Error::MissingVariants`] for an enumeration made
/// with [`new`](ConstantLabeling::new), and [`Error::NotAccepted`] if the label does not fit (a
/// float that is not finite, or an enumeration index that names no variant).
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantLabeling {
    name: String,
    kind: Kind,
    variants: Option<Variants>,
    value: Label,
}

impl ConstantLabeling {
    /// The labeling called `name` that gives `label` to every element; its kind is that of the
    /// label. For an enumeration use [`enumeration`](ConstantLabeling::enumeration).
    pub fn new(name: impl Into<String>, label: Label) -> ConstantLabeling {
        ConstantLabeling {
            name: name.into(),
            kind: label.kind(),
            variants: None,
            value: label,
        }
    }

    /// The labeling called `name` of an enumeration with the given `variants` that gives the
    /// variant number `index` to every element.
    pub fn enumeration(
        name: impl Into<String>,
        variants: Variants,
        index: u32,
    ) -> ConstantLabeling {
        ConstantLabeling {
            name: name.into(),
            kind: Kind::Enum,
            variants: Some(variants),
            value: Label::Enum(index),
        }
    }
}

impl<E> Labeling<E> for ConstantLabeling {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &Kind {
        &self.kind
    }

    fn variants(&self) -> Option<&Variants> {
        self.variants.as_ref()
    }

    fn label(&self, _ctx: &Context, _element: E) -> Option<Label> {
        Some(self.value.clone())
    }

    fn validate(&self) -> Result<(), Errors> {
        let variants_problem = variants_error(&self.kind, self.variants.as_ref());
        let misfit =
            variants_problem.is_none() && !fits(&self.kind, self.variants.as_ref(), &self.value);
        let problems: Vec<Error> = name_error(&self.name)
            .into_iter()
            .chain(variants_problem)
            .chain(misfit.then(|| Error::NotAccepted {
                name: self.name.clone(),
                kind: format!("{:?}", self.kind),
                label: format!("{:?}", self.value),
            }))
            .collect();
        Errors::from_list(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn gives_the_same_label_to_every_element() {
        let l = ConstantLabeling::new("w", Label::Int(7));
        assert_eq!(
            Labeling::<u64>::label(&l, &Context::default(), 0),
            Some(Label::Int(7))
        );
        assert_eq!(
            Labeling::<u64>::label(&l, &Context::default(), u64::MAX),
            Some(Label::Int(7))
        );
        assert_eq!(
            l.label(&Context::default(), (1u64, 2u64)),
            Some(Label::Int(7))
        );
        assert_eq!(Labeling::<u64>::name(&l), "w");
    }

    #[test]
    fn check_reports_a_label_that_does_not_fit() {
        let ab = || Variants::new(["a", "b"]);
        let wrong = [
            ConstantLabeling::new("x", Label::Float(f64::NAN)),
            ConstantLabeling::new("x", Label::Float(f64::INFINITY)),
            ConstantLabeling::enumeration("x", ab(), 2),
        ];
        for l in wrong {
            let errors = Labeling::<u64>::validate(&l).unwrap_err();
            assert!(matches!(errors.as_slice(), [Error::NotAccepted { .. }]));
        }
        let fine = ConstantLabeling::new("x", Label::Int(1));
        assert_eq!(Labeling::<u64>::validate(&fine), Ok(()));
        assert_eq!(
            Labeling::<u64>::validate(&ConstantLabeling::enumeration("x", ab(), 1)),
            Ok(())
        );
    }

    #[test]
    fn check_reports_bad_variants_once() {
        let problems = |l: ConstantLabeling| Labeling::<u64>::validate(&l).unwrap_err();
        let missing = ConstantLabeling::new("x", Label::Enum(0));
        assert_eq!(problems(missing).as_slice(), [Error::MissingVariants]);
        let none = ConstantLabeling::enumeration("x", Variants::new(Vec::<String>::new()), 0);
        assert_eq!(problems(none).as_slice(), [Error::NoVariants]);
    }

    #[test]
    fn formats_the_variant() {
        let l = ConstantLabeling::enumeration("x", Variants::new(["a", "b"]), 1);
        assert_eq!(Labeling::<u64>::format(&l, &Label::Enum(1)).unwrap(), "b");
    }

    #[test]
    fn works_through_pointers_and_trait_objects() {
        let l = ConstantLabeling::new("ok", Label::Bool(true));
        let boxed: Box<dyn Labeling<u64>> = Box::new(l.clone());
        assert_eq!(boxed.label(&Context::default(), 5), Some(Label::Bool(true)));
        let shared: Arc<dyn Labeling<(u64, u64)>> = Arc::new(l.clone());
        assert_eq!(
            shared.label(&Context::default(), (0, 1)),
            Some(Label::Bool(true))
        );
        assert_eq!(
            Labeling::<u64>::label(&&l, &Context::default(), 0),
            Some(Label::Bool(true))
        );
        assert_eq!(boxed.name(), "ok");
    }
}
