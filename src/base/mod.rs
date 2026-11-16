//! The foundations that `graphs` and `labels` share: the graph and its elements, the context a
//! graph is read under, and the generic list of errors. Everything here is independent of the
//! generators, the terms and the labelings; it depends only on `random`.

mod context;
mod element;
pub mod errors;
mod graph;

pub use context::Context;
pub(crate) use element::possible_edges;
pub use element::{Edge, Node};
pub use graph::{Graph, GraphDisplay};
