//! [`Labeling`]: a source of labels for the elements of a graph.

use std::sync::Arc;

use super::{Error, Errors, Kind, Label, Variants};
use crate::{Context, Graph};

/// A named, typed assignment of a [`Label`] to each element of some kind. It is generic over the
/// element type `E`, so one trait serves the nodes of a graph (the node id) and its edges
/// ([`Edge`](crate::Edge)); the labels may be stored or computed.
///
/// A labeling is read under a [`Context`]: a computed labeling has a stream and takes the master
/// seed from the context, and a stored one ignores it. Labelings are `Send + Sync`.
pub trait Labeling<E>: Send + Sync {
    /// The name of the labeling: an identifier.
    fn name(&self) -> &str;

    /// The kind of the labels.
    fn kind(&self) -> &Kind;

    /// The label of `element` under `ctx`, or `None` if it has none. A label returned fits the
    /// labeling: the [`kind`](Labeling::kind) accepts it and, for an enumeration, it names a
    /// variant.
    fn label(&self, ctx: &Context, element: E) -> Option<Label>;

    /// The variant names, if the labels are an enumeration: the label `Label::Enum(i)` is the
    /// `i`-th name.
    fn variants(&self) -> Option<&Variants> {
        None
    }

    /// The text of `label`, a label of this labeling: a scalar as it reads, an enumeration as its
    /// variant name. This is the only way to turn a label into text, since an enumeration label is
    /// an index. For any other label the text is unspecified, never a panic; `None` for an
    /// enumeration index that names no variant.
    fn format(&self, label: &Label) -> Option<String> {
        match label {
            Label::Bool(b) => Some(b.to_string()),
            Label::Int(i) => Some(i.to_string()),
            Label::Float(x) => Some(format!("{x:?}")),
            Label::Enum(index) => self.variants()?.get(*index).map(str::to_string),
            Label::String(s) => Some(s.to_string()),
        }
    }

    /// Reports every invalid parameter of the labeling, in order, as
    /// [`Term::validate`](crate::Term::validate) does for a term. Constructors of computed
    /// labelings do not fail; reading an invalid labeling never panics, but the labels are
    /// unspecified. The default accepts everything.
    fn validate(&self) -> Result<(), Errors> {
        Ok(())
    }

    /// Reports every way in which the labeling does not fit `graph`, which it was validated for.
    /// Only a stored labeling can fail to fit; the default accepts everything.
    fn resolve(&self, _graph: &dyn Graph) -> Result<(), Errors> {
        Ok(())
    }
}

/// Forwards [`Labeling`] through pointer types.
macro_rules! forward_labeling {
    ($($ptr:ty),*) => {$(
        impl<E, L: Labeling<E> + ?Sized> Labeling<E> for $ptr {
            fn name(&self) -> &str {
                (**self).name()
            }

            fn kind(&self) -> &Kind {
                (**self).kind()
            }

            fn label(&self, ctx: &Context, element: E) -> Option<Label> {
                (**self).label(ctx, element)
            }

            fn variants(&self) -> Option<&Variants> {
                (**self).variants()
            }

            fn format(&self, label: &Label) -> Option<String> {
                (**self).format(label)
            }

            fn validate(&self) -> Result<(), Errors> {
                (**self).validate()
            }

            fn resolve(&self, graph: &dyn Graph) -> Result<(), Errors> {
                (**self).resolve(graph)
            }
        }
    )*};
}

forward_labeling!(&L, Box<L>, Arc<L>);

/// [`Error::InvalidName`] if `name` is not an identifier: an ASCII letter or `_`, then ASCII
/// letters, digits or `_`.
pub(crate) fn name_error(name: &str) -> Option<Error> {
    let mut chars = name.chars();
    let valid = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    (!valid).then(|| Error::InvalidName(name.to_string()))
}

/// Whether `label` fits a labeling of `kind` with the given `variants` (those of an enumeration):
/// the kind accepts it and, for an enumeration, it is an index below the variants.
pub(crate) fn fits(kind: &Kind, variants: Option<&Variants>, label: &Label) -> bool {
    kind.accepts(label)
        && match label {
            Label::Bool(_) | Label::Int(_) | Label::Float(_) | Label::String(_) => true,
            Label::Enum(index) => variants.is_some_and(|v| v.get(*index).is_some()),
        }
}

/// [`Error::BadLabel`] for the first of `labels` that does not [fit](fits) a labeling of `kind`
/// with the given `variants`, with its position.
pub(crate) fn misfit<'a>(
    kind: &Kind,
    variants: Option<&Variants>,
    labels: impl IntoIterator<Item = &'a Label>,
) -> Option<Error> {
    labels
        .into_iter()
        .enumerate()
        .find(|(_, label)| !fits(kind, variants, label))
        .map(|(index, label)| Error::BadLabel {
            index,
            kind: format!("{kind:?}"),
            label: format!("{label:?}"),
        })
}

/// The problem with the `variants` of a labeling of `kind`: missing for an enumeration, or invalid.
pub(crate) fn variants_error(kind: &Kind, variants: Option<&Variants>) -> Option<Error> {
    match (kind, variants) {
        (Kind::Enum, None) => Some(Error::MissingVariants),
        (_, Some(variants)) => variants.validate().err(),
        (_, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_identifiers() {
        for name in ["color", "_x", "w2", "Weight", "a_b_c", "_"] {
            assert_eq!(name_error(name), None, "{name:?}");
        }
    }

    #[test]
    fn rejects_other_names() {
        for name in ["", "2x", "a-b", "a b", "a.b", "č", "x\u{301}"] {
            assert_eq!(
                name_error(name),
                Some(Error::InvalidName(name.to_string())),
                "{name:?}"
            );
        }
    }

    fn colors() -> Variants {
        Variants::new(["red", "green"])
    }

    /// A labeling of the given kind and variants, which has no labels.
    struct Empty(Kind, Option<Variants>);

    impl Labeling<u64> for Empty {
        fn name(&self) -> &str {
            "e"
        }

        fn kind(&self) -> &Kind {
            &self.0
        }

        fn label(&self, _ctx: &Context, _element: u64) -> Option<Label> {
            None
        }

        fn variants(&self) -> Option<&Variants> {
            self.1.as_ref()
        }
    }

    #[test]
    fn scalars_are_formatted_as_they_read() {
        let l = Empty(Kind::Int, None);
        let text = |label: Label| l.format(&label);
        assert_eq!(text(Label::Bool(false)).unwrap(), "false");
        assert_eq!(text(Label::Int(-42)).unwrap(), "-42");
        assert_eq!(text(Label::Float(1.0)).unwrap(), "1.0");
        assert_eq!(text(Label::Float(0.1)).unwrap(), "0.1");
        assert_eq!(text(Label::String("a b".into())).unwrap(), "a b");
    }

    #[test]
    fn an_enumeration_label_is_formatted_as_its_variant() {
        let l = Empty(Kind::Enum, Some(colors()));
        assert_eq!(l.format(&Label::Enum(1)).unwrap(), "green");
        assert_eq!(l.format(&Label::Enum(2)), None);
        let without = Empty(Kind::Enum, None);
        assert_eq!(without.format(&Label::Enum(0)), None);
    }

    #[test]
    fn misfit_finds_the_first_label_that_does_not_fit() {
        let v = colors();
        let labels = [
            Label::Enum(1),
            Label::Enum(0),
            Label::Enum(2),
            Label::Int(0),
        ];
        let found = misfit(&Kind::Enum, Some(&v), &labels);
        assert!(matches!(found, Some(Error::BadLabel { index: 2, .. })));
        assert_eq!(misfit(&Kind::Enum, Some(&v), &labels[..2]), None);
        let found = misfit(&Kind::Enum, None, &labels[..1]);
        assert!(matches!(found, Some(Error::BadLabel { index: 0, .. })));
        assert_eq!(misfit(&Kind::Int, None, &[Label::Int(1)]), None);
    }

    #[test]
    fn variants_and_format_are_forwarded_through_pointers() {
        let green = Label::Enum(1);
        let own = Empty(Kind::Enum, Some(colors()));
        let boxed: Box<dyn Labeling<u64>> = Box::new(Empty(Kind::Enum, Some(colors())));
        let shared: Arc<dyn Labeling<u64>> = Arc::new(Empty(Kind::Enum, Some(colors())));
        let by_ref = &own;
        assert_eq!(
            Labeling::<u64>::format(&by_ref, &green).as_deref(),
            Some("green")
        );
        assert_eq!(boxed.format(&green).as_deref(), Some("green"));
        assert_eq!(shared.format(&green).as_deref(), Some("green"));
        assert_eq!(boxed.variants(), Some(&colors()));
    }
}
