//! [`Complete`]: the complete graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node, possible_edges};

/// The complete graph on `order` nodes: every pair of distinct nodes is joined (both arcs if
/// directed). No self-loops.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Complete {
    order: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Complete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Complete").field(&self.order).finish()
    }
}

impl Complete {
    /// The complete graph with the given number of nodes.
    pub fn new(order: u64) -> Complete {
        Complete { order }
    }
}

impl Term for Complete {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    fn size(&self, ctx: &Context) -> u64 {
        let n = self.order;
        let arcs = n * n.saturating_sub(1);
        if ctx.directed { arcs } else { arcs / 2 }
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        Box::new(possible_edges(self.order, ctx.directed))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        u != v && self.has_node(ctx, u) && self.has_node(ctx, v)
    }

    /// The order must be at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn validate(&self) -> Result<(), Errors> {
        check_order("order", u128::from(self.order))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::Graph;
    use crate::testing::{build_graph, check_consistent, graph_pairs};

    #[test]
    fn undirected_k4() {
        let g = build_graph(Complete::new(4), false);
        assert_eq!(
            graph_pairs(&g),
            [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
        );
        assert_eq!((g.order(), g.size()), (4, 6));
    }

    #[test]
    fn directed_k3() {
        let g = build_graph(Complete::new(3), true);
        assert_eq!(
            graph_pairs(&g),
            [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)]
        );
        assert_eq!(g.size(), 6);
    }

    #[test]
    fn matches_edges_for_small_orders() {
        for n in 0..8 {
            check_consistent(&Complete::new(n), 9);
        }
    }
}
