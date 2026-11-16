//! The errors of the `graphs` module.

use std::fmt;

use super::Edge;

/// The errors of graphs and terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// An edge names a node that is not in `0..order`.
    EdgeOutOfRange {
        /// The offending edge, as given.
        edge: Edge,
        /// The order of the graph.
        order: u64,
    },
    /// A parameter has a value outside its allowed range.
    InvalidParam {
        /// The name of the parameter.
        name: &'static str,
        /// Why the value is not allowed.
        message: String,
    },
    /// A [`Spec`](crate::Spec) was built without an expression.
    MissingExpression,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EdgeOutOfRange { edge, order } => {
                write!(
                    f,
                    "edge ({}, {}) names a node out of range for a graph of order {order}",
                    edge.u, edge.v
                )
            }
            Error::InvalidParam { name, message } => {
                write!(f, "invalid parameter `{name}`: {message}")
            }
            Error::MissingExpression => write!(f, "the spec has no expression"),
        }
    }
}

impl std::error::Error for Error {}

/// Result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The errors found by [`Term::check`](crate::Term::check).
pub type Errors = crate::base::errors::Errors<Error>;
