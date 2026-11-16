//! The elements of a graph, [`Node`] and [`Edge`], and the possible edges.

use crate::random::Key;

/// A node; the nodes of a graph of order `n` are `0..n`.
pub type Node = u64;

/// An edge `(u, v)`: the arc `u -> v` if directed, the edge `{u, v}` if undirected (by convention
/// `u <= v`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    /// The first end: the tail if directed, the smaller end if undirected.
    pub u: Node,
    /// The second end: the head if directed, the larger end if undirected.
    pub v: Node,
}

impl Edge {
    /// The edge `(u, v)` for the given directedness: [`Edge::directed`] or [`Edge::undirected`].
    pub fn new(directed: bool, u: Node, v: Node) -> Edge {
        if directed {
            Edge::directed(u, v)
        } else {
            Edge::undirected(u, v)
        }
    }

    /// A directed edge `u -> v`, kept as given.
    pub fn directed(u: Node, v: Node) -> Edge {
        Edge { u, v }
    }

    /// An undirected edge, with the ends in ascending order.
    pub fn undirected(u: Node, v: Node) -> Edge {
        if u <= v {
            Edge { u, v }
        } else {
            Edge { u: v, v: u }
        }
    }
}

/// Folds as the pair `(u, v)`. An undirected edge must be normalized first: `(u, v)` and `(v, u)`
/// hash differently.
impl Key for Edge {
    fn fold(&self, state: u64) -> u64 {
        (self.u, self.v).fold(state)
    }
}

/// The non-loop edges on `order` nodes, in canonical order: `u < v` if undirected, `u != v` if
/// directed.
pub(crate) fn possible_edges(order: u64, directed: bool) -> impl Iterator<Item = Edge> {
    (0..order).flat_map(move |u| {
        let first = if directed { 0 } else { u + 1 };
        (first..order)
            // Needed only if directed.
            .filter(move |&v| v != u)
            .map(move |v| Edge::directed(u, v))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_edge_folds_like_its_pair_of_ends() {
        // Reference value of `(3, 4)` under the state 9.
        assert_eq!(Edge::directed(3, 4).fold(9), 0xee43_fb1c_b276_c6f3);
        assert_eq!(Edge::directed(3, 4).fold(9), (3u64, 4u64).fold(9));
        assert_ne!(Edge::directed(3, 4).fold(9), Edge::directed(4, 3).fold(9));
    }

    #[test]
    fn undirected_edge_is_normalized() {
        assert_eq!(Edge::undirected(3, 1), Edge::undirected(1, 3));
        assert_eq!(Edge::undirected(3, 1).u, 1);
    }
}
