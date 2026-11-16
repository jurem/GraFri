//! [`Kind`]: what a [`Label`] can be.

use super::Label;

/// What a [`Label`] can be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// [`Label::Bool`].
    Bool,
    /// [`Label::Int`].
    Int,
    /// [`Label::Float`], finite.
    Float,
    /// [`Label::Enum`]: an index into the variant names, which the labeling keeps.
    Enum,
    /// [`Label::String`].
    String,
}

impl Kind {
    /// Whether `label` is of this kind: the right variant of [`Label`], and a finite float. Whether
    /// an enumeration index names a variant is for the labeling to say.
    pub fn accepts(&self, label: &Label) -> bool {
        *self == label.kind() && !matches!(label, Label::Float(x) if !x.is_finite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_matching_variant() {
        let cases = [
            (Kind::Bool, Label::Bool(true), true),
            (Kind::Int, Label::Int(-3), true),
            (Kind::Float, Label::Float(0.5), true),
            (Kind::Enum, Label::Enum(1), true),
            (Kind::String, Label::String("x".into()), true),
            (Kind::Bool, Label::Int(1), false),
            (Kind::Int, Label::Float(1.0), false),
            (Kind::Float, Label::Int(1), false),
            (Kind::Enum, Label::Int(0), false),
            (Kind::String, Label::Enum(0), false),
        ];
        for (kind, label, expected) in cases {
            assert_eq!(kind.accepts(&label), expected, "{kind:?} {label:?}");
        }
    }

    #[test]
    fn floats_must_be_finite() {
        for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(!Kind::Float.accepts(&Label::Float(x)));
        }
        assert!(Kind::Float.accepts(&Label::Float(-0.0)));
    }
}
