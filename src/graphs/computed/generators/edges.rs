//! [`Edges`]: a graph given by its edges.

use std::fmt;
use std::sync::OnceLock;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Error, Errors, Node};

/// A graph given by its node pairs. Undirected, a pair is an unordered edge; directed, it is the
/// arc `u -> v`. Repeated edges count once, and self-loops are kept.
///
/// The pairs are kept sorted and without repeats, as arcs; the undirected graph is derived from
/// them on first use and kept.
#[derive(Clone)]
pub struct Edges {
    order: u64,
    /// The pairs as arcs, sorted, without repeats.
    directed_edges: Vec<Edge>,
    /// The same pairs as undirected edges, sorted, without repeats.
    undirected_edges: OnceLock<Vec<Edge>>,
}

/// Shown as its constructor call.
impl fmt::Debug for Edges {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pairs: Vec<(Node, Node)> = self.directed_edges.iter().map(|e| (e.u, e.v)).collect();
        f.debug_tuple("Edges")
            .field(&self.order)
            .field(&pairs)
            .finish()
    }
}

impl Edges {
    /// The graph with `order` nodes and the edges `pairs`.
    pub fn new(order: u64, pairs: impl IntoIterator<Item = (Node, Node)>) -> Edges {
        let mut directed_edges: Vec<Edge> = pairs
            .into_iter()
            .map(|(s, t)| Edge::directed(s, t))
            .collect();
        directed_edges.sort_unstable();
        directed_edges.dedup();
        Edges {
            order,
            directed_edges,
            undirected_edges: OnceLock::new(),
        }
    }

    /// The edges in canonical order, directed or undirected as set by `ctx`.
    fn canonical_edges(&self, ctx: &Context) -> &[Edge] {
        if ctx.directed {
            &self.directed_edges
        } else {
            self.undirected_edges.get_or_init(|| {
                let mut edges: Vec<Edge> = self
                    .directed_edges
                    .iter()
                    .map(|e| Edge::undirected(e.u, e.v))
                    .collect();
                edges.sort_unstable();
                edges.dedup();
                edges
            })
        }
    }
}

impl Term for Edges {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    fn size(&self, ctx: &Context) -> u64 {
        self.canonical_edges(ctx).len() as u64
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        Box::new(self.canonical_edges(ctx).iter().copied())
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let probe = Edge::new(ctx.directed, u, v);
        self.canonical_edges(ctx).binary_search(&probe).is_ok()
    }

    /// The order must be at most [`MAX_ORDER`](crate::MAX_ORDER), and a pair with a node
    /// `>= order` is an error.
    fn validate(&self) -> Result<(), Errors> {
        let order = self.order;
        let order_ok = check_order("order", u128::from(order));
        let out_of_range = self
            .directed_edges
            .iter()
            .filter(|e| e.u >= order || e.v >= order)
            .map(|&edge| Err(Error::EdgeOutOfRange { edge, order }.into()));
        Errors::collect(std::iter::once(order_ok).chain(out_of_range))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{check_consistent, edges, pairs_of};

    #[test]
    fn undirected_sorts_normalizes_and_dedups() {
        let g = edges(4, &[(3, 2), (1, 0), (2, 0), (0, 1), (2, 2)]);
        assert_eq!(pairs_of(&g, false), [(0, 1), (0, 2), (2, 2), (2, 3)]);
        let ctx = Context::undirected();
        assert_eq!((g.order(&ctx), g.size(&ctx)), (4, 4));
    }

    #[test]
    fn directed_keeps_the_direction() {
        let g = edges(3, &[(1, 0), (0, 1), (1, 0), (2, 2)]);
        assert_eq!(pairs_of(&g, true), [(0, 1), (1, 0), (2, 2)]);
    }

    #[test]
    fn one_expression_serves_both_directed_and_undirected() {
        let g = edges(3, &[(0, 1), (1, 0)]);
        assert_eq!(g.size(&Context::undirected()), 1);
        assert_eq!(g.size(&Context::directed()), 2);
    }

    #[test]
    fn rejects_nodes_out_of_range() {
        let errors = Edges::new(3, [(0, 3)]).validate().unwrap_err();
        assert_eq!(
            errors.as_slice(),
            [Error::EdgeOutOfRange {
                edge: Edge::directed(0, 3),
                order: 3
            }]
        );
        // Every offending edge is reported once, even with two bad nodes.
        let errors = Edges::new(2, [(5, 5), (0, 5), (7, 1)])
            .validate()
            .unwrap_err();
        let out_of_range = |u, v| Error::EdgeOutOfRange {
            edge: Edge::directed(u, v),
            order: 2,
        };
        assert_eq!(
            errors.as_slice(),
            [out_of_range(0, 5), out_of_range(5, 5), out_of_range(7, 1)]
        );
        assert_eq!(Edges::new(0, []).validate(), Ok(()));
    }

    #[test]
    fn has_edge_and_size_match_edges() {
        check_consistent(&edges(4, &[(0, 1), (2, 0), (3, 3)]), 6);
    }
}
