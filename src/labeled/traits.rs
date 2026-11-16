//! The traits of labeled graphs: the two kinds of labeling, and reading labels by name.

use crate::graphs::{Edge, Node};
use crate::labels::{Label, Labeling};

/// A [`Labeling`] of the nodes of a graph.
pub trait NodeLabeling: Labeling<Node> {}
impl<T: Labeling<Node> + ?Sized> NodeLabeling for T {}

/// A [`Labeling`] of the edges of a graph.
pub trait EdgeLabeling: Labeling<Edge> {}
impl<T: Labeling<Edge> + ?Sized> EdgeLabeling for T {}

/// A graph that carries node labels, found by name.
pub trait NodeLabels {
    /// The node labelings, in the order they were added.
    fn node_labelings(&self) -> Box<dyn Iterator<Item = &dyn NodeLabeling> + '_>;

    /// The label called `name` of the node `v`; `None` if there is no such labeling or `v` is not a
    /// node of the graph.
    fn node_label(&self, name: &str, v: Node) -> Option<Label>;
}

/// A graph that carries edge labels, found by name.
pub trait EdgeLabels {
    /// The edge labelings, in the order they were added.
    fn edge_labelings(&self) -> Box<dyn Iterator<Item = &dyn EdgeLabeling> + '_>;

    /// The label called `name` of the edge `u -> v` (`u -- v`, in either order, if undirected);
    /// `None` if there is no such labeling or the graph has no such edge.
    fn edge_label(&self, name: &str, u: Node, v: Node) -> Option<Label>;
}
