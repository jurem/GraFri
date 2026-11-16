//! [`Wheel`]: the wheel graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The wheel on `order` nodes: node 0 is the hub, joined to each of the other nodes, which form a
/// cycle (the rim). `order` counts the hub. Directed, the spokes go from the hub, and the rim arcs
/// are `i -> i + 1`, the last one back to 1. No self-loops; one node or none have no edges, and two
/// nodes have the single spoke `0 - 1`. A rim of two nodes is a single edge if undirected, and two
/// arcs if directed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Wheel {
    order: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Wheel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Wheel").field(&self.order).finish()
    }
}

impl Wheel {
    /// The wheel with the given number of nodes, the hub included.
    pub fn new(order: u64) -> Wheel {
        Wheel { order }
    }
}

impl Term for Wheel {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    fn size(&self, ctx: &Context) -> u64 {
        let n = self.order;
        match n {
            0 | 1 => 0,
            2 => 1,
            3 if !ctx.directed => 3,
            n => (n - 1) * 2,
        }
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let n = self.order;
        // `0 < i`, so the spokes are canonical in both cases.
        let spokes = (1..n).map(|i| Edge::directed(0, i));
        // The rim `1..n` has `n - 1` nodes; a cycle needs at least two.
        if n < 3 {
            Box::new(spokes)
        } else if ctx.directed {
            // The arcs `i -> i + 1`, then `n - 1 -> 1`, which sorts last.
            let rim = (1..n - 1).map(|i| Edge::directed(i, i + 1));
            Box::new(
                spokes
                    .chain(rim)
                    .chain(std::iter::once(Edge::directed(n - 1, 1))),
            )
        } else if n == 3 {
            Box::new(spokes.chain(std::iter::once(Edge::directed(1, 2))))
        } else {
            // The closing edge `1 - (n - 1)` sorts right after `1 - 2`.
            let rest = (2..n - 1).map(|i| Edge::directed(i, i + 1));
            Box::new(
                spokes
                    .chain(std::iter::once(Edge::directed(1, 2)))
                    .chain(std::iter::once(Edge::directed(1, n - 1)))
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
            // From the hub to anyone; on the rim, to the next node, wrapping to 1.
            u == 0 || (v != 0 && v == if u == n - 1 { 1 } else { u + 1 })
        } else if u == 0 || v == 0 {
            true
        } else {
            // Two distinct rim nodes, so `n >= 3`.
            let gap = u.abs_diff(v);
            gap == 1 || gap == n - 2
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
    use crate::graphs::computed::generators::Complete;
    use crate::testing::{build_graph, check_consistent, graph_pairs};

    #[test]
    fn undirected_w5_lists_the_closing_edge_in_order() {
        let g = build_graph(Wheel::new(5), false);
        assert_eq!(
            graph_pairs(&g),
            [
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (1, 4),
                (2, 3),
                (3, 4)
            ]
        );
        assert_eq!((g.order(), g.size()), (5, 8));
    }

    #[test]
    fn directed_w5_goes_around_the_rim() {
        let g = build_graph(Wheel::new(5), true);
        assert_eq!(
            graph_pairs(&g),
            [
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 1)
            ]
        );
        assert_eq!(g.size(), 8);
    }

    #[test]
    fn matches_the_definition_for_small_orders() {
        for n in 0..9 {
            check_consistent(&Wheel::new(n), 10);
            for directed in [false, true] {
                let ctx = Context::new(directed);
                // Independent of the code under test: the definition on all pairs.
                let rim = |u: u64, v: u64| u != 0 && v != 0 && (u % (n - 1)) + 1 == v;
                let expected: Vec<Edge> = (0..n)
                    .flat_map(|u| (0..n).map(move |v| (u, v)))
                    .filter(|&(u, v)| {
                        u != v
                            && (u == 0
                                || (!directed && v == 0)
                                || rim(u, v)
                                || (!directed && rim(v, u)))
                    })
                    .map(|(u, v)| Edge::new(directed, u, v))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                assert_eq!(
                    Wheel::new(n).edges(&ctx).collect::<Vec<_>>(),
                    expected,
                    "n={n} directed={directed}"
                );
            }
        }
    }

    #[test]
    fn wheel_of_four_is_the_complete_graph_if_undirected() {
        let ctx = Context::undirected();
        assert_eq!(
            Wheel::new(4).edges(&ctx).collect::<Vec<_>>(),
            Complete::new(4).edges(&ctx).collect::<Vec<_>>()
        );
    }
}
