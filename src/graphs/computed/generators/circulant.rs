//! [`Circulant`]: the circulant graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The circulant graph on `order` nodes with the given jumps: node `i` is joined to
/// `(i + j) mod order` for each jump `j`. Directed, `j` and `order - j` are different jumps;
/// undirected, they are the same jump, and for `j = order / 2` each edge is listed once. A jump
/// is taken modulo `order`, and one that is a multiple of `order` is ignored (no self-loops). No
/// jumps give no edges. The jumps are kept reduced, sorted, without repeats.
#[derive(Clone, PartialEq, Eq)]
pub struct Circulant {
    order: u64,
    jumps: Vec<u64>,
}

/// Shown as its constructor call.
impl fmt::Debug for Circulant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Circulant")
            .field(&self.order)
            .field(&self.jumps)
            .finish()
    }
}

impl Circulant {
    /// The circulant graph with `order` nodes and the given jumps, each taken modulo `order`.
    pub fn new(order: u64, jumps: impl IntoIterator<Item = u64>) -> Circulant {
        let mut jumps: Vec<u64> = jumps
            .into_iter()
            .filter(|_| order > 0)
            .map(|j| j % order)
            .filter(|&j| j > 0)
            .collect();
        jumps.sort_unstable();
        jumps.dedup();
        Circulant { order, jumps }
    }

    /// The distinct jumps that make edges under the given directedness, sorted: if undirected,
    /// each as `min(j, order - j)`.
    fn effective_jumps(&self, directed: bool) -> Vec<u64> {
        if directed {
            return self.jumps.clone();
        }
        let n = self.order;
        let mut jumps: Vec<u64> = self.jumps.iter().map(|&j| j.min(n - j)).collect();
        jumps.sort_unstable();
        jumps.dedup();
        jumps
    }

    /// The numbers of edges that each undirected jump makes: `order`, or half of it for the half
    /// jump, which makes each edge from both of its ends.
    fn undirected_counts(&self) -> impl Iterator<Item = u64> {
        let n = self.order;
        self.effective_jumps(false)
            .into_iter()
            .map(move |j| if j == n - j { n / 2 } else { n })
    }
}

impl Term for Circulant {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    /// O(1) for k jumps if directed, O(k log k) if not.
    fn size(&self, ctx: &Context) -> u64 {
        let n = self.order;
        if ctx.directed {
            return self.jumps.len() as u64 * n;
        }
        self.undirected_counts().sum()
    }

    /// Collects and sorts the edges, in O(nk) memory for k jumps.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let (n, directed) = (self.order, ctx.directed);
        let jumps = self.effective_jumps(directed);
        let mut edges: Vec<Edge> = (0..n)
            .flat_map(|i| {
                jumps
                    .iter()
                    .map(move |&j| Edge::new(directed, i, (i + j) % n))
            })
            .collect();
        edges.sort_unstable();
        // Undirected, the jump `order / 2` makes each edge from both of its ends.
        edges.dedup();
        Box::new(edges.into_iter())
    }

    /// O(log k) for k jumps.
    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let n = self.order;
        if !self.has_node(ctx, u) || !self.has_node(ctx, v) {
            return false;
        }
        let has = |j: u64| self.jumps.binary_search(&j).is_ok();
        if ctx.directed {
            // The jump from `u` to `v`, going up and wrapping around.
            has(if v > u { v - u } else { n - (u - v) })
        } else {
            let gap = u.abs_diff(v);
            has(gap) || has(n - gap)
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
    use crate::graphs::computed::generators::{Complete, Cycle};
    use crate::testing::{build_graph, check_consistent, graph_pairs};

    #[test]
    fn directed_arcs_go_up_by_each_jump() {
        let g = build_graph(Circulant::new(5, [2]), true);
        assert_eq!(graph_pairs(&g), [(0, 2), (1, 3), (2, 4), (3, 0), (4, 1)]);
    }

    #[test]
    fn jump_and_its_mirror_are_one_jump_if_undirected() {
        let one = Circulant::new(7, [2]);
        let both = Circulant::new(7, [2, 5]);
        let u = Context::undirected();
        assert_eq!(
            one.edges(&u).collect::<Vec<_>>(),
            both.edges(&u).collect::<Vec<_>>()
        );
        let d = Context::directed();
        assert_eq!(one.size(&d), 7);
        assert_eq!(both.size(&d), 14);
    }

    #[test]
    fn the_half_jump_lists_each_edge_once() {
        let g = build_graph(Circulant::new(6, [3]), false);
        assert_eq!(graph_pairs(&g), [(0, 3), (1, 4), (2, 5)]);
        assert_eq!(g.size(), 3);
        let d = build_graph(Circulant::new(6, [3]), true);
        assert_eq!(d.size(), 6);
    }

    #[test]
    fn jumps_are_reduced_sorted_and_without_repeats() {
        // 13 is 4 modulo 9, and 0 and 9 are multiples of 9.
        let c = Circulant::new(9, [4, 1, 13, 2, 9, 0]);
        assert_eq!(c.jumps, [1, 2, 4]);
        assert_eq!(Circulant::new(5, [7]), Circulant::new(5, [2]));
        assert!(Circulant::new(0, [1, 2]).jumps.is_empty());
    }

    #[test]
    fn cycle_and_complete_are_circulants() {
        for directed in [false, true] {
            let ctx = Context::new(directed);
            for n in 3..9 {
                assert_eq!(
                    Circulant::new(n, [1]).edges(&ctx).collect::<Vec<_>>(),
                    Cycle::new(n).edges(&ctx).collect::<Vec<_>>()
                );
                assert_eq!(
                    Circulant::new(n, 1..n).edges(&ctx).collect::<Vec<_>>(),
                    Complete::new(n).edges(&ctx).collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn matches_the_definition_for_small_orders() {
        for n in 0..9 {
            let all = [vec![], vec![1], vec![2], vec![1, 3], vec![2, 3, 5], vec![4]];
            let more = [vec![0], vec![7], vec![1, 6], vec![10, 0, 13]];
            for jumps in all.into_iter().chain(more) {
                let c = Circulant::new(n, jumps.clone());
                assert_eq!(c.validate(), Ok(()));
                check_consistent(&c, 10);
                for directed in [false, true] {
                    let ctx = Context::new(directed);
                    // Independent of the code under test: the definition `v = u + j (mod n)` on all
                    // pairs.
                    let expected: Vec<Edge> = (0..n)
                        .flat_map(|u| (0..n).map(move |v| (u, v)))
                        .filter(|&(u, v)| {
                            let hit = |a: u64, b: u64| jumps.iter().any(|&j| (a + j) % n == b);
                            u != v && (hit(u, v) || (!directed && hit(v, u)))
                        })
                        .map(|(u, v)| Edge::new(directed, u, v))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    assert_eq!(
                        c.edges(&ctx).collect::<Vec<_>>(),
                        expected,
                        "n={n} jumps={jumps:?} directed={directed}"
                    );
                }
            }
        }
    }
}
