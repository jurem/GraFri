//! [`Path`]: the path graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The path on `order` nodes: node `i` is joined to `i + 1`. Directed, the arcs are `i -> i + 1`.
/// No self-loops; one node or none have no edges.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Path {
    order: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Path").field(&self.order).finish()
    }
}

impl Path {
    /// The path with the given number of nodes.
    pub fn new(order: u64) -> Path {
        Path { order }
    }
}

impl Term for Path {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    fn size(&self, _ctx: &Context) -> u64 {
        self.order.saturating_sub(1)
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        // `i < i + 1`, so the same edges are canonical in both cases.
        Box::new((0..self.size(ctx)).map(|i| Edge::directed(i, i + 1)))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let (low, high) = if ctx.directed || u <= v {
            (u, v)
        } else {
            (v, u)
        };
        low.checked_add(1) == Some(high) && self.has_node(ctx, high)
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
    fn undirected_and_directed_p4() {
        for directed in [false, true] {
            let g = build_graph(Path::new(4), directed);
            assert_eq!(graph_pairs(&g), [(0, 1), (1, 2), (2, 3)]);
            assert_eq!((g.order(), g.size()), (4, 3));
        }
    }

    #[test]
    fn matches_the_definition_for_small_orders() {
        for n in 0..8 {
            check_consistent(&Path::new(n), 9);
            for directed in [false, true] {
                let ctx = Context::new(directed);
                let expected: Vec<Edge> = (0..n)
                    .flat_map(|u| (0..n).map(move |v| Edge::new(directed, u, v)))
                    .filter(|e| e.v == e.u + 1 || (!directed && e.u == e.v + 1))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                assert_eq!(Path::new(n).edges(&ctx).collect::<Vec<_>>(), expected);
            }
        }
    }
}
