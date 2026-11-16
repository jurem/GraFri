//! [`EdgeList`]: a graph stored as its sorted edge list.

use crate::graphs::{Edge, Error, Errors, Graph, Node};

/// A graph stored as its sorted edge list, with its own directedness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeList {
    order: u64,
    directed: bool,
    edges: Vec<Edge>,
}

impl EdgeList {
    /// A graph with `order` nodes and the edges `pairs`.
    /// Undirected pairs are normalized, the edges sorted, and repeats counted once.
    ///
    /// # Errors
    /// [`Error::EdgeOutOfRange`] for each pair with a node `>= order`, all in one [`Errors`].
    pub fn new(
        order: u64,
        directed: bool,
        pairs: impl IntoIterator<Item = (Node, Node)>,
    ) -> Result<EdgeList, Errors> {
        let mut edges = Vec::new();
        let mut errors = Vec::new();
        for (u, v) in pairs {
            if u >= order || v >= order {
                let edge = Edge::directed(u, v);
                errors.push(Err(Error::EdgeOutOfRange { edge, order }.into()));
            } else {
                edges.push(Edge::new(directed, u, v));
            }
        }
        Errors::collect(errors)?;
        edges.sort_unstable();
        edges.dedup();
        Ok(EdgeList {
            order,
            directed,
            edges,
        })
    }

    /// The graph with `order` nodes and no edges.
    pub fn empty(order: u64, directed: bool) -> EdgeList {
        EdgeList {
            order,
            directed,
            edges: Vec::new(),
        }
    }

    /// Materializes any graph.
    pub fn from_graph(graph: &(impl Graph + ?Sized)) -> EdgeList {
        EdgeList {
            order: graph.order(),
            directed: graph.is_directed(),
            edges: graph.edges().collect(),
        }
    }

    /// The edges, in canonical order.
    pub fn as_slice(&self) -> &[Edge] {
        &self.edges
    }
}

impl Graph for EdgeList {
    fn order(&self) -> u64 {
        self.order
    }

    fn is_directed(&self) -> bool {
        self.directed
    }

    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        Box::new(self.edges.iter().copied())
    }

    fn size(&self) -> u64 {
        self.edges.len() as u64
    }

    fn has_edge(&self, u: Node, v: Node) -> bool {
        let probe = Edge::new(self.directed, u, v);
        self.edges.binary_search(&probe).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(g: &EdgeList) -> Vec<(Node, Node)> {
        g.edges().map(|e| (e.u, e.v)).collect()
    }

    #[test]
    fn sorts_and_normalizes_undirected() {
        let g = EdgeList::new(4, false, [(3, 2), (1, 0), (2, 0)]).unwrap();
        assert_eq!(pairs(&g), [(0, 1), (0, 2), (2, 3)]);
        assert_eq!((g.order(), g.size()), (4, 3));
    }

    #[test]
    fn keeps_direction_when_directed() {
        let g = EdgeList::new(3, true, [(2, 0), (0, 2), (1, 0)]).unwrap();
        assert_eq!(pairs(&g), [(0, 2), (1, 0), (2, 0)]);
    }

    #[test]
    fn repeated_edges_count_once() {
        let g = EdgeList::new(3, false, [(0, 1), (1, 0), (0, 1)]).unwrap();
        assert_eq!(pairs(&g), [(0, 1)]);
        let d = EdgeList::new(3, true, [(0, 1), (1, 0), (0, 1)]).unwrap();
        assert_eq!(pairs(&d), [(0, 1), (1, 0)]);
    }

    #[test]
    fn reports_every_out_of_range_edge_in_the_order_given() {
        let errors = EdgeList::new(3, false, [(0, 3), (1, 2), (7, 0), (9, 9)]).unwrap_err();
        let edge = |u, v| Error::EdgeOutOfRange {
            edge: Edge::directed(u, v),
            order: 3,
        };
        assert_eq!(errors.as_slice(), [edge(0, 3), edge(7, 0), edge(9, 9)]);
    }

    #[test]
    fn has_edge_respects_directedness() {
        let u = EdgeList::new(3, false, [(0, 2)]).unwrap();
        assert!(u.has_edge(2, 0) && u.has_edge(0, 2) && !u.has_edge(0, 1));
        let d = EdgeList::new(3, true, [(0, 2)]).unwrap();
        assert!(d.has_edge(0, 2) && !d.has_edge(2, 0));
    }
}
