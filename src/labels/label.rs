//! [`Label`]: what a node or an edge carries.

use std::sync::Arc;

use super::Kind;

/// A label: the datum that a node or an edge carries under a name. Its [`Kind`](super::Kind)
/// decides which labels are valid ([`Kind::accepts`](super::Kind::accepts)). `PartialEq` but not
/// `Eq` or `Ord`, because of the float.
#[derive(Debug, Clone, PartialEq)]
pub enum Label {
    /// A boolean.
    Bool(bool),
    /// A signed integer.
    Int(i64),
    /// A float. NaN and infinities are not valid labels.
    Float(f64),
    /// An enumeration label: the index into the variant names of its labeling.
    Enum(u32),
    /// A UTF-8 string.
    String(Arc<str>),
}

impl Label {
    /// The kind this label belongs to.
    pub fn kind(&self) -> Kind {
        match self {
            Label::Bool(_) => Kind::Bool,
            Label::Int(_) => Kind::Int,
            Label::Float(_) => Kind::Float,
            Label::Enum(_) => Kind::Enum,
            Label::String(_) => Kind::String,
        }
    }
}
