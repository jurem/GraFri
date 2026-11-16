//! Labeled graphs: a graph with node labelings and edge labelings. It sits above `graphs` and
//! `labels`, which stay independent of each other.

mod traits;
mod wrapper;

pub use traits::{EdgeLabeling, EdgeLabels, NodeLabeling, NodeLabels};
pub use wrapper::Labeled;
