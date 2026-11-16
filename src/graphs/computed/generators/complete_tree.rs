//! [`CompleteTree`]: the complete k-ary tree.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Error, Errors, Node};

/// The k-ary tree on `order` nodes, filled level by level: node 0 is the root, and node `i > 0` has
/// the parent `(i - 1) / arity`. Directed, the arcs go from the parent to the child. Every level is
/// full except possibly the last, which is filled from the left.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CompleteTree {
    arity: u64,
    order: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for CompleteTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CompleteTree")
            .field(&self.arity)
            .field(&self.order)
            .finish()
    }
}

impl CompleteTree {
    /// The tree with at most `arity` children per node and `order` nodes.
    pub fn new(arity: u64, order: u64) -> CompleteTree {
        CompleteTree { arity, order }
    }
}

impl Term for CompleteTree {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    fn size(&self, _ctx: &Context) -> u64 {
        self.order.saturating_sub(1)
    }

    fn edges(&self, _ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        // The parents do not decrease with `i`, so these are in ascending order,
        // and `parent < i`, so the same edges are canonical in both cases.
        let arity = self.arity;
        Box::new((1..self.order).map(move |i| Edge::directed((i - 1) / arity, i)))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let (parent, child) = if ctx.directed || u <= v {
            (u, v)
        } else {
            (v, u)
        };
        child >= 1 && self.has_node(ctx, child) && parent == (child - 1) / self.arity
    }

    /// `arity` must be at least 1, and the order at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn validate(&self) -> Result<(), Errors> {
        let arity = if self.arity >= 1 {
            Ok(())
        } else {
            Err(Error::InvalidParam {
                name: "arity",
                message: "must be at least 1, got 0".to_string(),
            }
            .into())
        };
        Errors::collect([arity, check_order("order", u128::from(self.order))])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::Graph;
    use crate::testing::{build_graph, check_consistent, graph_pairs};

    #[test]
    fn binary_tree_with_seven_nodes() {
        for directed in [false, true] {
            let g = build_graph(CompleteTree::new(2, 7), directed);
            assert_eq!(
                graph_pairs(&g),
                [(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)]
            );
            assert_eq!((g.order(), g.size()), (7, 6));
        }
    }

    #[test]
    fn a_last_level_that_is_not_full() {
        let g = build_graph(CompleteTree::new(3, 6), false);
        assert_eq!(graph_pairs(&g), [(0, 1), (0, 2), (0, 3), (1, 4), (1, 5)]);
    }

    #[test]
    fn arity_one_is_the_path_and_tiny_orders_have_no_edges() {
        let path = build_graph(CompleteTree::new(1, 5), true);
        assert_eq!(graph_pairs(&path), [(0, 1), (1, 2), (2, 3), (3, 4)]);
        for n in [0, 1] {
            let g = build_graph(CompleteTree::new(2, n), false);
            assert_eq!((g.order(), g.size(), g.edges().count()), (n, 0, 0));
        }
    }

    #[test]
    fn matches_the_definition_for_small_trees() {
        for arity in 1..5 {
            for n in 0..14 {
                let t = CompleteTree::new(arity, n);
                check_consistent(&t, 15);
                // Independent of the code under test: the children of `j` are `arity * j + 1 ..=
                // arity * j + arity`.
                let expected: Vec<(Node, Node)> = (0..n)
                    .flat_map(|j| {
                        (arity * j + 1..=arity * j + arity)
                            .filter(|&c| c < n)
                            .map(move |c| (j, c))
                    })
                    .collect();
                for directed in [false, true] {
                    let got: Vec<_> = t
                        .edges(&Context::new(directed))
                        .map(|e| (e.u, e.v))
                        .collect();
                    assert_eq!(got, expected, "arity={arity} n={n}");
                }
            }
        }
    }

    #[test]
    fn directed_arcs_go_from_the_parent() {
        let t = CompleteTree::new(2, 7);
        let d = Context::directed();
        assert!(t.has_edge(&d, 1, 4) && !t.has_edge(&d, 4, 1));
        let u = Context::undirected();
        assert!(t.has_edge(&u, 4, 1) && t.has_edge(&u, 1, 4));
        assert!(!t.has_edge(&u, 3, 4) && !t.has_edge(&u, 0, 0) && !t.has_edge(&u, 3, 7));
    }

    #[test]
    fn check_requires_a_positive_arity() {
        assert_eq!(CompleteTree::new(1, 5).validate(), Ok(()));
        let errors = CompleteTree::new(0, 5).validate().unwrap_err();
        assert!(matches!(
            errors.as_slice(),
            [Error::InvalidParam { name: "arity", .. }]
        ));
    }
}
