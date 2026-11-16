//! [`Star`]: the star graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The star with the given number of leaves: node 0 is the hub, joined to each of the leaves
/// `1..=leaves`, so there are `leaves + 1` nodes. Directed, the arcs go from the hub to the leaves.
/// No self-loops; no leaves is a single node.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Star {
    leaves: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Star {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Star").field(&self.leaves).finish()
    }
}

impl Star {
    /// The star with the given number of leaves.
    pub fn new(leaves: u64) -> Star {
        Star { leaves }
    }
}

impl Term for Star {
    fn order(&self, _ctx: &Context) -> u64 {
        self.leaves + 1
    }

    fn size(&self, _ctx: &Context) -> u64 {
        self.leaves
    }

    fn edges(&self, _ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        // `0 < i`, so the same edges are canonical in both cases.
        Box::new((1..=self.leaves).map(|i| Edge::directed(0, i)))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let (hub, leaf) = if ctx.directed || u == 0 {
            (u, v)
        } else {
            (v, u)
        };
        hub == 0 && leaf != 0 && self.has_node(ctx, leaf)
    }

    /// The order, `leaves + 1`, must be at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn validate(&self) -> Result<(), Errors> {
        check_order("leaves", u128::from(self.leaves) + 1)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::graphs::Graph;
    use crate::graphs::computed::generators::CompleteBipartite;
    use crate::testing::{build_graph, check_consistent, check_order_limit, graph_pairs};

    #[test]
    fn star_with_four_leaves() {
        for directed in [false, true] {
            let g = build_graph(Star::new(4), directed);
            assert_eq!(graph_pairs(&g), [(0, 1), (0, 2), (0, 3), (0, 4)]);
            assert_eq!((g.order(), g.size()), (5, 4));
        }
    }

    #[test]
    fn matches_the_definition_for_small_stars() {
        for leaves in 0..8 {
            let n = leaves + 1;
            check_consistent(&Star::new(leaves), 9);
            for directed in [false, true] {
                let ctx = Context::new(directed);
                // Independent of the code under test: the definition on all pairs.
                let expected: Vec<Edge> = (0..n)
                    .flat_map(|u| (0..n).map(move |v| (u, v)))
                    .filter(|&(u, v)| u != v && (u == 0 || (!directed && v == 0)))
                    .map(|(u, v)| Edge::new(directed, u, v))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                assert_eq!(Star::new(leaves).edges(&ctx).collect::<Vec<_>>(), expected);
            }
        }
    }

    #[test]
    fn is_the_complete_bipartite_graph_with_one_node_on_one_side() {
        for directed in [false, true] {
            let ctx = Context::new(directed);
            for leaves in 0..6 {
                assert_eq!(
                    Star::new(leaves).edges(&ctx).collect::<Vec<_>>(),
                    CompleteBipartite::new(1, leaves)
                        .edges(&ctx)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn the_order_is_limited() {
        check_order_limit(|order| Star::new(order - 1));
    }
}
