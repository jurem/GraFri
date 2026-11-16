//! The [`Graph`] trait.

use std::fmt;
use std::sync::Arc;

use super::{Edge, Node};

/// A finite graph on the nodes `0..order()`, directed or not. Self-loops are allowed.
pub trait Graph {
    /// The number of nodes.
    fn order(&self) -> u64;

    /// The nodes, in ascending order.
    fn nodes(&self) -> std::ops::Range<Node> {
        0..self.order()
    }

    /// Whether `v` is a node of the graph.
    fn has_node(&self, v: Node) -> bool {
        v < self.order()
    }

    /// The number of edges. O(size) by default.
    fn size(&self) -> u64 {
        self.edges().count() as u64
    }

    /// The edges in ascending order, without repeats (not checked), the same every time. An
    /// undirected edge has `u <= v`.
    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_>;

    /// Whether the graph has the edge `u -> v` (`u -- v`, in either order, if undirected). O(size)
    /// by default.
    fn has_edge(&self, u: Node, v: Node) -> bool {
        let probe = Edge::new(self.is_directed(), u, v);
        self.edges().find(|e| *e >= probe) == Some(probe)
    }

    /// Whether the edges are directed.
    fn is_directed(&self) -> bool;

    /// A view of the graph as text: its order and size, and with
    /// [`with_edges`](GraphDisplay::with_edges) its edges too.
    fn display(&self) -> GraphDisplay<'_>
    where
        Self: Sized,
    {
        GraphDisplay::new(self)
    }
}

/// The text view of a graph, made by [`Graph::display`]: `graph(3, 2)` for a graph of order 3 and
/// size 2, and `digraph(3, 2)` if it is directed. With its edges: `graph(3, 2): 0-1 1-2`, or
/// `digraph(3, 2): 0->1 1->2`.
pub struct GraphDisplay<'a> {
    graph: &'a dyn Graph,
    edges: bool,
}

impl<'a> GraphDisplay<'a> {
    /// The view of `graph` without its edges. For a graph that is not a trait object,
    /// [`Graph::display`] is the short form.
    pub fn new(graph: &'a dyn Graph) -> GraphDisplay<'a> {
        GraphDisplay {
            graph,
            edges: false,
        }
    }

    /// The same view, followed by the edges, which are written as the graph is read.
    pub fn with_edges(self) -> Self {
        GraphDisplay {
            edges: true,
            ..self
        }
    }
}

impl fmt::Display for GraphDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let graph = self.graph;
        let (keyword, arrow) = if graph.is_directed() {
            ("digraph", "->")
        } else {
            ("graph", "-")
        };
        write!(f, "{keyword}({}, {})", graph.order(), graph.size())?;
        if self.edges {
            write!(f, ":")?;
            for e in graph.edges() {
                write!(f, " {}{arrow}{}", e.u, e.v)?;
            }
        }
        Ok(())
    }
}

/// Forwards [`Graph`] through pointer types.
macro_rules! forward_graph {
    ($($ptr:ty),*) => {$(
        impl<G: Graph + ?Sized> Graph for $ptr {
            fn order(&self) -> u64 {
                (**self).order()
            }

            fn nodes(&self) -> std::ops::Range<Node> {
                (**self).nodes()
            }

            fn has_node(&self, v: Node) -> bool {
                (**self).has_node(v)
            }

            fn size(&self) -> u64 {
                (**self).size()
            }

            fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
                (**self).edges()
            }

            fn has_edge(&self, u: Node, v: Node) -> bool {
                (**self).has_edge(u, v)
            }

            fn is_directed(&self) -> bool {
                (**self).is_directed()
            }
        }
    )*};
}

forward_graph!(&G, Box<G>, Arc<G>);

#[cfg(test)]
mod tests {
    use super::*;

    /// A graph given by its order and its sorted edges.
    struct Listed(u64, bool, Vec<Edge>);

    impl Graph for Listed {
        fn order(&self) -> u64 {
            self.0
        }

        fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
            Box::new(self.2.iter().copied())
        }

        fn is_directed(&self) -> bool {
            self.1
        }
    }

    #[test]
    fn display_lists_the_edges_with_the_arrow_of_the_graph() {
        let edges = |directed| vec![Edge::new(directed, 0, 1), Edge::new(directed, 1, 2)];
        let undirected = Listed(3, false, edges(false));
        assert_eq!(undirected.display().to_string(), "graph(3, 2)");
        assert_eq!(
            undirected.display().with_edges().to_string(),
            "graph(3, 2): 0-1 1-2"
        );
        let directed = Listed(3, true, edges(true));
        assert_eq!(directed.display().to_string(), "digraph(3, 2)");
        assert_eq!(
            directed.display().with_edges().to_string(),
            "digraph(3, 2): 0->1 1->2"
        );
        let empty = Listed(0, false, vec![]);
        assert_eq!(empty.display().with_edges().to_string(), "graph(0, 0):");
        // A trait object has no `display`, but the view can be made from it.
        let object: &dyn Graph = &directed;
        assert_eq!(GraphDisplay::new(object).to_string(), "digraph(3, 2)");
    }
}
