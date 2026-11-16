//! Graphs: the errors of graphs are here; `computed` has the
//! terms that describe a graph, and `stored` the graphs kept in memory. Both depend only on
//! the rest of this module.

pub mod computed;
mod error;
pub mod stored;

pub(crate) use crate::base::possible_edges;
pub use crate::base::{Edge, Graph, Node};
pub use error::{Error, Errors, Result};
