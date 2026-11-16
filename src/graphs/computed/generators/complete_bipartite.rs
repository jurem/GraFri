//! [`CompleteBipartite`]: the complete bipartite graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The complete bipartite graph on `left + right` nodes: `0..left` is one side and
/// `left..left + right` the other, and every node of one side is joined to every node of the other.
/// Directed, the arcs go from the left side to the right. No self-loops.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CompleteBipartite {
    left: u64,
    right: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for CompleteBipartite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CompleteBipartite")
            .field(&self.left)
            .field(&self.right)
            .finish()
    }
}

impl CompleteBipartite {
    /// The complete bipartite graph with sides of the given sizes.
    pub fn new(left: u64, right: u64) -> CompleteBipartite {
        CompleteBipartite { left, right }
    }
}

impl Term for CompleteBipartite {
    fn order(&self, _ctx: &Context) -> u64 {
        self.left + self.right
    }

    fn size(&self, _ctx: &Context) -> u64 {
        self.left * self.right
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        // `u < v`, so the same edges are canonical in both cases.
        let (left, end) = (self.left, self.order(ctx));
        Box::new((0..left).flat_map(move |u| (left..end).map(move |v| Edge::directed(u, v))))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let (u, v) = if ctx.directed || u <= v {
            (u, v)
        } else {
            (v, u)
        };
        u < self.left && self.left <= v && self.has_node(ctx, v)
    }

    /// The order, `left + right`, must be at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn validate(&self) -> Result<(), Errors> {
        let order = u128::from(self.left) + u128::from(self.right);
        check_order("left, right", order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::Graph;
    use crate::testing::{build_graph, check_consistent, check_order_limit, graph_pairs};

    #[test]
    fn k_2_3_undirected_and_directed() {
        for directed in [false, true] {
            let g = build_graph(CompleteBipartite::new(2, 3), directed);
            assert_eq!(
                graph_pairs(&g),
                [(0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4)]
            );
            assert_eq!((g.order(), g.size()), (5, 6));
        }
    }

    #[test]
    fn matches_the_definition_for_small_sides() {
        for left in 0..5 {
            for right in 0..5 {
                let e = CompleteBipartite::new(left, right);
                check_consistent(&e, 10);
                let n = left + right;
                for directed in [false, true] {
                    let ctx = Context::new(directed);
                    // Independent of the code under test: the definition on all pairs.
                    let expected: Vec<(Node, Node)> = (0..n)
                        .flat_map(|u| (0..n).map(move |v| (u, v)))
                        .filter(|&(u, v)| {
                            if directed {
                                u < left && v >= left
                            } else {
                                u < v && u < left && v >= left
                            }
                        })
                        .collect();
                    let got: Vec<_> = e.edges(&ctx).map(|e| (e.u, e.v)).collect();
                    assert_eq!(got, expected, "{left},{right} directed={directed}");
                }
            }
        }
    }

    #[test]
    fn the_order_is_limited() {
        check_order_limit(|order| CompleteBipartite::new(order / 2, order - order / 2));
        check_order_limit(|order| CompleteBipartite::new(order, 0));
        assert!(
            CompleteBipartite::new(u64::MAX, u64::MAX)
                .validate()
                .is_err()
        );
    }
}
