//! The error type of the `labels` module.

use std::fmt;

/// The errors of the `labels` module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A label name is not an identifier (an ASCII letter or `_`, then ASCII
    /// letters, digits or `_`).
    InvalidName(String),
    /// An enumeration has no variants.
    NoVariants,
    /// An enumeration has more variants than a label can index.
    TooManyVariants,
    /// A variant name of an enumeration is empty.
    EmptyVariant,
    /// Two variants of an enumeration have the same name.
    RepeatedVariant(String),
    /// A labeling of an enumeration was made without its variants.
    MissingVariants,
    /// A label in a sequence does not fit the kind of the labeling.
    BadLabel {
        /// The position of the first bad label.
        index: usize,
        /// The kind that rejected it, as text.
        kind: String,
        /// The offending label, as text.
        label: String,
    },
    /// A label of a keyed labeling does not fit the kind of the labeling.
    BadKeyedLabel {
        /// The key of the first bad label, as text.
        key: String,
        /// The kind that rejected it, as text.
        kind: String,
        /// The offending label, as text.
        label: String,
    },
    /// A key occurs twice in the entries of a keyed labeling; the key, as text.
    DuplicateKey(String),
    /// A dense labeling does not have one label per node.
    LabelCount {
        /// The number of nodes of the graph.
        expected: u64,
        /// The number of labels.
        found: usize,
    },
    /// A key of a keyed labeling is not an element of the graph.
    UnknownElement(String),
    /// A label name is defined twice for the same kind of element.
    DuplicateName(String),
    /// The range of an integer labeling has no numbers (`start >= end`).
    EmptyRange,
    /// The range of a float labeling is not usable: a bound or the width is not
    /// finite, or `start >= end`.
    InvalidFloatRange,
    /// A probability is not in `[0, 1]`.
    InvalidProbability,
    /// A label does not fit the kind of the label it was given for.
    NotAccepted {
        /// The name of the label.
        name: String,
        /// The kind that rejected it, as text.
        kind: String,
        /// The offending label, as text.
        label: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidName(name) => write!(
                f,
                "`{name}` is not a label name (a letter or `_`, then letters, digits or `_`)"
            ),
            Error::NoVariants => write!(f, "an enumeration needs at least one variant"),
            Error::TooManyVariants => write!(f, "too many variants in an enumeration"),
            Error::EmptyVariant => write!(f, "a variant name cannot be empty"),
            Error::RepeatedVariant(name) => write!(f, "the variant `{name}` is repeated"),
            Error::MissingVariants => write!(
                f,
                "a labeling of an enumeration needs its variants: make it with `enumeration`"
            ),
            Error::BadLabel { index, kind, label } => {
                write!(f, "label {index} ({label}) does not fit the kind {kind}")
            }
            Error::BadKeyedLabel { key, kind, label } => {
                write!(
                    f,
                    "the label of {key} ({label}) does not fit the kind {kind}"
                )
            }
            Error::DuplicateKey(key) => write!(f, "the key {key} occurs more than once"),
            Error::LabelCount { expected, found } => {
                write!(
                    f,
                    "the graph has {expected} nodes but the labeling has {found} labels"
                )
            }
            Error::UnknownElement(element) => {
                write!(f, "{element} is not an element of the graph")
            }
            Error::DuplicateName(name) => write!(f, "the label `{name}` is defined twice"),
            Error::EmptyRange => write!(f, "the range of integers is empty"),
            Error::InvalidFloatRange => write!(
                f,
                "the range of floats needs finite bounds, a finite width, and start < end"
            ),
            Error::InvalidProbability => write!(f, "a probability must be in [0, 1]"),
            Error::NotAccepted { name, kind, label } => {
                write!(f, "{label} does not fit the kind {kind} of `{name}`")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The errors found by [`Labeling::validate`](super::Labeling::validate) and
/// [`Labeling::resolve`](super::Labeling::resolve).
pub type Errors = crate::base::errors::Errors<Error>;
