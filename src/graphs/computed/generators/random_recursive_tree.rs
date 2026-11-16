//! [`RandomRecursiveTree`]: the random recursive tree, decided node by node.

use std::fmt;
use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};
use crate::random::{self, Key};

/// The random recursive tree: node 0 is the root, and every node `i > 0` is attached to a parent
/// chosen uniformly from `0..i`. Directed, the arcs go from the parent to the child.
///
/// The tree is a function of the order, the `stream` and the seed of the context: the same values
/// always give the same tree, and another stream or seed gives an independent sample. Every node
/// chooses its parent on its own. The rule that decides a parent is fixed by the reproducibility
/// contract.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RandomRecursiveTree {
    order: u64,
    stream: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for RandomRecursiveTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RandomRecursiveTree")
            .field(&self.order)
            .field(&self.stream)
            .finish()
    }
}

impl RandomRecursiveTree {
    /// The random recursive tree on `order` nodes, on the random stream number
    /// `stream`.
    pub fn new(order: u64, stream: u64) -> RandomRecursiveTree {
        RandomRecursiveTree { order, stream }
    }

    /// The seed that the nodes are hashed with under `ctx`.
    fn seed(&self, ctx: &Context) -> u64 {
        random::stream_seed(ctx.seed, self.stream)
    }

    /// The parent of the node `child >= 1`: `bounded(child.fold(seed), child)`, where `seed` is
    /// `stream_seed(ctx.seed, stream)`.
    fn parent(&self, ctx: &Context, child: Node) -> Node {
        random::bounded(child.fold(self.seed(ctx)), child)
    }

    /// The edges in canonical order, the same for both directedness.
    fn canonical_edges(&self, ctx: &Context) -> Arc<Vec<Edge>> {
        let params = [self.order, self.stream, 0]; // The third slot is unused.
        ctx.cached("random_recursive_tree", params, || {
            let mut edges: Vec<Edge> = (1..self.order)
                .map(|child| Edge::directed(self.parent(ctx, child), child))
                .collect();
            edges.sort_unstable();
            edges
        })
    }
}

impl Term for RandomRecursiveTree {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    /// O(1).
    fn size(&self, _ctx: &Context) -> u64 {
        self.order.saturating_sub(1)
    }

    /// The parents are not in order, so this sorts them once per graph, in O(order log order) time
    /// and O(order) memory, and keeps the list in the context's cache.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let edges = self.canonical_edges(ctx);
        // The closure keeps the cached list alive.
        Box::new((0..edges.len()).map(move |i| edges[i]))
    }

    /// O(1).
    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let (parent, child) = if ctx.directed || u <= v {
            (u, v)
        } else {
            (v, u)
        };
        child >= 1 && self.has_node(ctx, child) && parent == self.parent(ctx, child)
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
    use crate::testing::{build_graph_in, check_consistent};

    fn ctx(directed: bool, seed: u64) -> Context {
        Context::new(directed).with_seed(seed)
    }

    fn parents_of(order: u64, stream: u64, seed: u64) -> Vec<Node> {
        let t = RandomRecursiveTree::new(order, stream);
        let c = ctx(true, seed);
        (1..order).map(|i| t.parent(&c, i)).collect()
    }

    #[test]
    fn parents_follow_the_reference_vectors() {
        // The parents of nodes 1, 2, ... from an independent Python implementation of `mix`,
        // `bounded` and `stream_seed`.
        assert_eq!(parents_of(12, 2, 3), [0, 0, 0, 3, 1, 1, 6, 7, 7, 9, 3]);
        assert_eq!(parents_of(8, 0, 0), [0, 1, 2, 3, 1, 0, 5]);
        assert_eq!(parents_of(10, 5, 42), [0, 0, 1, 2, 1, 1, 2, 4, 4]);
    }

    #[test]
    fn edges_are_the_sorted_parent_edges() {
        let t = RandomRecursiveTree::new(12, 2);
        for directed in [false, true] {
            let got: Vec<_> = t.edges(&ctx(directed, 3)).map(|e| (e.u, e.v)).collect();
            let mut expected: Vec<(Node, Node)> = [0, 0, 0, 3, 1, 1, 6, 7, 7, 9, 3]
                .iter()
                .zip(1..)
                .map(|(&p, c)| (p, c))
                .collect();
            expected.sort_unstable();
            assert_eq!(got, expected);
        }
    }

    #[test]
    fn is_a_tree() {
        for seed in 0..5 {
            for n in 0..30u64 {
                let t = RandomRecursiveTree::new(n, 1);
                let c = ctx(false, seed);
                assert_eq!(t.size(&c), n.saturating_sub(1));
                assert_eq!(t.edges(&c).count() as u64, n.saturating_sub(1));
                // Every node but the root has exactly one earlier neighbor, its
                // parent, so following the parents leads to the root.
                for v in 1..n {
                    let earlier = (0..v).filter(|&u| t.has_edge(&c, u, v)).count();
                    assert_eq!(earlier, 1, "seed={seed} n={n} v={v}");
                }
            }
        }
    }

    #[test]
    fn matches_has_edge_and_size() {
        check_consistent(&RandomRecursiveTree::new(10, 3), 12);
    }

    #[test]
    fn same_context_same_tree_other_seed_or_stream_other_tree() {
        let list = |stream, seed| {
            RandomRecursiveTree::new(40, stream)
                .edges(&ctx(false, seed))
                .collect::<Vec<_>>()
        };
        let a = list(5, 1);
        assert_eq!(a, list(5, 1));
        assert_ne!(a, list(6, 1));
        assert_ne!(a, list(5, 2));
    }

    #[test]
    fn works_as_a_graph_and_its_edges_are_cached_per_graph() {
        let g = build_graph_in(
            RandomRecursiveTree::new(30, 4),
            Context::undirected().with_seed(9),
        );
        let first: Vec<_> = g.edges().collect();
        assert_eq!(first, g.edges().collect::<Vec<_>>());
        assert_eq!(first, g.clone().edges().collect::<Vec<_>>());
        assert_eq!((g.order(), g.size()), (30, 29));
    }
}
