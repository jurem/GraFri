//! [`Cycle`]: the cycle graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The cycle on `order` nodes: node `i` is joined to `i + 1`, and the last node to node 0.
/// Directed, the arcs are `i -> (i + 1) mod order`. No self-loops, so one node or none have no
/// edges; two nodes have the single edge `0 - 1` if undirected, and both arcs if directed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cycle {
    order: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Cycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Cycle").field(&self.order).finish()
    }
}

impl Cycle {
    /// The cycle with the given number of nodes.
    pub fn new(order: u64) -> Cycle {
        Cycle { order }
    }
}

impl Term for Cycle {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    fn size(&self, ctx: &Context) -> u64 {
        match self.order {
            0 | 1 => 0,
            2 if !ctx.directed => 1,
            n => n,
        }
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let n = self.order;
        if n < 2 {
            Box::new(std::iter::empty())
        } else if ctx.directed {
            // The arcs `i -> i + 1`, then `n - 1 -> 0`, which sorts last.
            let path = (0..n - 1).map(|i| Edge::directed(i, i + 1));
            Box::new(path.chain(std::iter::once(Edge::directed(n - 1, 0))))
        } else if n == 2 {
            Box::new(std::iter::once(Edge::directed(0, 1)))
        } else {
            // The closing edge `0 - (n - 1)` sorts right after `0 - 1`.
            let rest = (1..n - 1).map(|i| Edge::directed(i, i + 1));
            Box::new(
                std::iter::once(Edge::directed(0, 1))
                    .chain(std::iter::once(Edge::directed(0, n - 1)))
                    .chain(rest),
            )
        }
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let n = self.order;
        if u == v || !self.has_node(ctx, u) || !self.has_node(ctx, v) {
            return false;
        }
        if ctx.directed {
            v == (u + 1) % n
        } else {
            let gap = u.abs_diff(v);
            gap == 1 || gap == n - 1
        }
    }

    /// The order must be at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn validate(&self) -> Result<(), Errors> {
        check_order("order", u128::from(self.order))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::graphs::Graph;
    use crate::testing::{build_graph, check_consistent, graph_pairs};

    #[test]
    fn undirected_c5_lists_the_closing_edge_in_order() {
        let g = build_graph(Cycle::new(5), false);
        assert_eq!(graph_pairs(&g), [(0, 1), (0, 4), (1, 2), (2, 3), (3, 4)]);
        assert_eq!((g.order(), g.size()), (5, 5));
    }

    #[test]
    fn directed_c4_goes_around() {
        let g = build_graph(Cycle::new(4), true);
        assert_eq!(graph_pairs(&g), [(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(g.size(), 4);
    }

    #[test]
    fn matches_the_definition_for_small_orders() {
        for n in 0..9 {
            check_consistent(&Cycle::new(n), 10);
            for directed in [false, true] {
                let ctx = Context::new(directed);
                // Independent of the code under test: the definition `v = u + 1 (mod n)` on all
                // pairs.
                let expected: Vec<Edge> = (0..n)
                    .flat_map(|u| (0..n).map(move |v| (u, v)))
                    .filter(|&(u, v)| {
                        u != v && ((u + 1) % n == v || (!directed && (v + 1) % n == u))
                    })
                    .map(|(u, v)| Edge::new(directed, u, v))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                assert_eq!(
                    Cycle::new(n).edges(&ctx).collect::<Vec<_>>(),
                    expected,
                    "n={n} directed={directed}"
                );
            }
        }
    }
}
