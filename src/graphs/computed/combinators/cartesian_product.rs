//! [`CartesianProduct`]: two terms multiplied.

use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::combinators::loops;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The Cartesian product of two graphs: the nodes are the pairs `(u, v)`, numbered
/// `u * b.order() + v`. The node `(u, v)` is joined to `(u', v)` for each edge `u ~ u'` of `a`, and
/// to `(u, v')` for each edge `v ~ v'` of `b`. Directed, the arcs follow the arcs of the operands.
///
/// Self-loops are kept: a loop of `a` at `u` gives a loop at `(u, v)` for every `v`, and a loop of
/// `b` at `v` gives a loop at `(u, v)` for every `u`. A loop that both make counts once.
#[derive(Debug, Clone)]
pub struct CartesianProduct(Arc<dyn Term>, Arc<dyn Term>);

impl CartesianProduct {
    /// The Cartesian product of `a` and `b`.
    pub fn new(a: impl Term + 'static, b: impl Term + 'static) -> CartesianProduct {
        CartesianProduct(a.shared(), b.shared())
    }
}

impl Term for CartesianProduct {
    fn order(&self, ctx: &Context) -> u64 {
        self.0.order(ctx) * self.1.order(ctx)
    }

    /// Reads the operands' edges once to find their loops, so it is O(m1 + m2).
    fn size(&self, ctx: &Context) -> u64 {
        let (n1, n2) = (self.0.order(ctx), self.1.order(ctx));
        let (m1, m2) = (self.0.size(ctx), self.1.size(ctx));
        let twice = loops(&*self.0, ctx) * loops(&*self.1, ctx);
        m1 * n2 + n1 * m2 - twice
    }

    /// Collects and sorts the edges, in O(m1 * n2 + n1 * m2) memory.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let (n1, n2) = (self.0.order(ctx), self.1.order(ctx));
        let directed = ctx.directed;
        let at = move |u: Node, v: Node| u * n2 + v;
        let along_a = self
            .0
            .edges(ctx)
            .flat_map(move |e| (0..n2).map(move |v| Edge::new(directed, at(e.u, v), at(e.v, v))));
        let along_b = (0..n1).flat_map(move |u| {
            self.1
                .edges(ctx)
                .map(move |e| Edge::new(directed, at(u, e.u), at(u, e.v)))
        });
        let mut edges: Vec<Edge> = along_a.chain(along_b).collect();
        edges.sort_unstable();
        // A loop of both operands is made twice.
        edges.dedup();
        Box::new(edges.into_iter())
    }

    /// O(1) in the operands' `has_edge`.
    fn has_edge(&self, ctx: &Context, x: Node, y: Node) -> bool {
        if !self.has_node(ctx, x) || !self.has_node(ctx, y) {
            return false;
        }
        // Both nodes exist, so `n2 > 0`.
        let n2 = self.1.order(ctx);
        let ((ux, vx), (uy, vy)) = ((x / n2, x % n2), (y / n2, y % n2));
        (vx == vy && self.0.has_edge(ctx, ux, uy)) || (ux == uy && self.1.has_edge(ctx, vx, vy))
    }

    fn validate(&self) -> Result<(), Errors> {
        Errors::collect([self.0.validate(), self.1.validate()])
    }

    /// The product of the two orders must be at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn resolve(&self, ctx: &Context) -> Result<(), Errors> {
        // The orders are read only after the operands passed: their product may overflow before.
        Errors::collect([self.0.resolve(ctx), self.1.resolve(ctx)])?;
        let orders = u128::from(self.0.order(ctx)) * u128::from(self.1.order(ctx));
        check_order("orders", orders)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::graphs::Error;
    use crate::graphs::computed::MAX_ORDER;
    use crate::graphs::computed::generators::{Complete, Cycle, Grid, Hypercube, Path, Torus};
    use crate::testing::{check_consistent, edges, pairs_of};

    fn same_graph(a: &impl Term, b: &impl Term) {
        for directed in [false, true] {
            let ctx = Context::new(directed);
            assert_eq!(a.order(&ctx), b.order(&ctx));
            assert_eq!(
                a.edges(&ctx).collect::<Vec<_>>(),
                b.edges(&ctx).collect::<Vec<_>>(),
                "directed={directed}"
            );
        }
    }

    #[test]
    fn path_times_path_is_a_grid() {
        for rows in 0..5 {
            for cols in 0..5 {
                same_graph(
                    &CartesianProduct::new(Path::new(rows), Path::new(cols)),
                    &Grid::new(rows, cols),
                );
            }
        }
    }

    #[test]
    fn cycle_times_cycle_is_a_torus() {
        for rows in 0..6 {
            for cols in 0..6 {
                same_graph(
                    &CartesianProduct::new(Cycle::new(rows), Cycle::new(cols)),
                    &Torus::new(rows, cols),
                );
            }
        }
    }

    #[test]
    fn hypercube_times_k2_is_the_next_hypercube() {
        for d in 0..4 {
            same_graph(
                &CartesianProduct::new(Hypercube::new(d), Complete::new(2)),
                &Hypercube::new(d + 1),
            );
        }
    }

    #[test]
    fn directed_arcs_follow_the_operands() {
        let a = edges(2, &[(1, 0)]);
        let b = edges(3, &[(0, 1), (2, 1)]);
        let p = CartesianProduct::new(a, b);
        // The arcs of `a` go from row 1 to row 0 (`3 -> 0`, `4 -> 1`, `5 -> 2`); the arcs of
        // `b` go `0 -> 1` and `2 -> 1` within each row.
        assert_eq!(
            pairs_of(&p, true),
            [(0, 1), (2, 1), (3, 0), (3, 4), (4, 1), (5, 2), (5, 4)]
        );
    }

    #[test]
    fn a_loop_of_both_operands_counts_once() {
        let a = edges(2, &[(0, 0)]);
        let b = edges(2, &[(0, 0), (0, 1)]);
        let p = CartesianProduct::new(a, b);
        assert_eq!(
            pairs_of(&p, false),
            [(0, 0), (0, 1), (1, 1), (2, 2), (2, 3)]
        );
        let ctx = Context::undirected();
        assert_eq!((p.order(&ctx), p.size(&ctx)), (4, 5));
        check_consistent(&p, 5);
    }

    #[test]
    fn matches_the_definition() {
        let a = edges(3, &[(0, 1), (2, 0), (1, 1)]);
        let b = edges(3, &[(1, 0), (1, 2), (0, 0)]);
        let p = CartesianProduct::new(a.clone(), b.clone());
        check_consistent(&p, 10);
        for directed in [false, true] {
            let ctx = Context::new(directed);
            let (ea, eb): (Vec<Edge>, Vec<Edge>) =
                (a.edges(&ctx).collect(), b.edges(&ctx).collect());
            // Independent of the code under test: the definition on all pairs, with the operands'
            // edges looked up in lists.
            let expected: Vec<Edge> = (0..9)
                .flat_map(|x| (0..9).map(move |y| (x, y)))
                .filter(|&(x, y)| {
                    let ((ux, vx), (uy, vy)) = ((x / 3, x % 3), (y / 3, y % 3));
                    let joined = |list: &[Edge], s, t| list.contains(&Edge::new(directed, s, t));
                    (vx == vy && joined(&ea, ux, uy)) || (ux == uy && joined(&eb, vx, vy))
                })
                .map(|(x, y)| Edge::new(directed, x, y))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            assert_eq!(p.edges(&ctx).collect::<Vec<_>>(), expected);
        }
    }

    #[test]
    fn empty_operands_give_an_empty_graph() {
        let ctx = Context::undirected();
        for p in [
            CartesianProduct::new(Complete::new(0), Complete::new(3)),
            CartesianProduct::new(Complete::new(3), Complete::new(0)),
        ] {
            assert_eq!(
                (p.order(&ctx), p.size(&ctx), p.edges(&ctx).count()),
                (0, 0, 0)
            );
            assert!(!p.has_edge(&ctx, 0, 0));
        }
    }

    #[test]
    fn check_requires_the_orders_to_multiply() {
        let ctx = Context::undirected();
        let fine = CartesianProduct::new(Complete::new(3), Complete::new(4));
        assert_eq!(fine.check(&ctx), Ok(()));
        let at_the_limit = CartesianProduct::new(edges(MAX_ORDER, &[]), edges(1, &[]));
        assert_eq!(at_the_limit.check(&ctx), Ok(()));
        let overflow = CartesianProduct::new(edges(MAX_ORDER, &[]), edges(2, &[]));
        assert_eq!(overflow.validate(), Ok(()));
        let errors = overflow.check(&ctx).unwrap_err();
        assert!(matches!(
            errors.as_slice(),
            [Error::InvalidParam { name: "orders", .. }]
        ));
        // The errors of the operands stop `check` before the one about the orders.
        let both = CartesianProduct::new(edges(MAX_ORDER, &[(0, u64::MAX)]), edges(2, &[(0, 5)]));
        assert_eq!(both.check(&ctx).unwrap_err().as_slice().len(), 2);
        assert_eq!(both.resolve(&ctx).unwrap_err().as_slice().len(), 1);
    }

    #[test]
    fn nested_orders_are_checked_before_they_are_read() {
        let ctx = Context::undirected();
        let inner = CartesianProduct::new(edges(MAX_ORDER, &[]), edges(MAX_ORDER, &[]));
        let outer = CartesianProduct::new(inner, edges(MAX_ORDER, &[]));
        assert!(matches!(
            outer.check(&ctx).unwrap_err().as_slice(),
            [Error::InvalidParam { name: "orders", .. }]
        ));
    }
}
