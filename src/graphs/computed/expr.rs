//! [`Expr`]: combining terms by method call and with operators.

use std::fmt;
use std::ops::{Add, BitAnd, BitOr, Mul, Not, Sub};
use std::sync::Arc;

use crate::graphs::computed::Term;
use crate::graphs::computed::combinators::{
    CartesianProduct, Complement, Difference, DisjointUnion, Intersection, Reverse, Union,
};
use crate::graphs::computed::generators::Edges;

/// Wraps a term so that it can be combined by method call and with operators. The leaves are
/// the generators, wrapped with [`Expr::new`]; a raw term also works as the right operand.
///
/// Operators, with Rust's precedence: `a | b` union, `a & b` intersection, `a - b` difference,
/// `a + b` disjoint union, `a * b` Cartesian product, `!a` complement. The left operand must be an
/// `Expr`; the right one is anything that converts into one. A `&Expr` works on either side.
#[derive(Clone)]
pub struct Expr(Arc<dyn Term>);

impl Expr {
    /// An [`Expr`] that holds `term`.
    pub fn new(term: impl Term + 'static) -> Expr {
        Expr(term.shared())
    }

    /// At least `order` nodes: the union with the empty graph on `order` nodes.
    pub fn with_min_order(self, order: u64) -> Expr {
        self.union(Edges::new(order, std::iter::empty()))
    }

    /// The complement ([`Complement`]).
    pub fn complement(self) -> Expr {
        Expr(Complement::new(self.0).shared())
    }

    /// The reverse: every arc turned around ([`Reverse`]).
    pub fn reverse(self) -> Expr {
        Expr(Reverse::new(self.0).shared())
    }

    /// The union with `other` ([`Union`]).
    pub fn union(self, other: impl Into<Expr>) -> Expr {
        Expr(Union::new(self.0, other.into().0).shared())
    }

    /// The intersection with `other` ([`Intersection`]).
    pub fn intersection(self, other: impl Into<Expr>) -> Expr {
        Expr(Intersection::new(self.0, other.into().0).shared())
    }

    /// The edges not in `other` ([`Difference`]).
    pub fn difference(self, other: impl Into<Expr>) -> Expr {
        Expr(Difference::new(self.0, other.into().0).shared())
    }

    /// The disjoint union with `other` ([`DisjointUnion`]).
    pub fn disjoint_union(self, other: impl Into<Expr>) -> Expr {
        Expr(DisjointUnion::new(self.0, other.into().0).shared())
    }

    /// The Cartesian product with `other` ([`CartesianProduct`]).
    pub fn cartesian_product(self, other: impl Into<Expr>) -> Expr {
        Expr(CartesianProduct::new(self.0, other.into().0).shared())
    }

    /// The term itself, unchecked.
    pub fn term(&self) -> Arc<dyn Term> {
        Arc::clone(&self.0)
    }
}

/// Shown as the term it holds.
impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Expr").field(&self.0).finish()
    }
}

/// Any term converts into an expr.
impl<E: Term + 'static> From<E> for Expr {
    fn from(term: E) -> Expr {
        Expr::new(term)
    }
}

/// An expr converts into a copy of itself.
impl From<&Expr> for Expr {
    fn from(expr: &Expr) -> Expr {
        expr.clone()
    }
}

/// A binary operator on exprs: the left side by value or by reference, the right side anything
/// that converts into an expr.
macro_rules! binary_operator {
    ($trait:ident, $function:ident, $method:ident) => {
        impl<R: Into<Expr>> $trait<R> for Expr {
            type Output = Expr;

            fn $function(self, rhs: R) -> Expr {
                self.$method(rhs)
            }
        }

        impl<R: Into<Expr>> $trait<R> for &Expr {
            type Output = Expr;

            fn $function(self, rhs: R) -> Expr {
                self.clone().$method(rhs)
            }
        }
    };
}

binary_operator!(BitOr, bitor, union);
binary_operator!(BitAnd, bitand, intersection);
binary_operator!(Sub, sub, difference);
binary_operator!(Add, add, disjoint_union);
binary_operator!(Mul, mul, cartesian_product);

impl Not for Expr {
    type Output = Expr;

    fn not(self) -> Expr {
        self.complement()
    }
}

impl Not for &Expr {
    type Output = Expr;

    fn not(self) -> Expr {
        self.clone().complement()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::computed::generators::{Complete, Gnp};
    use crate::graphs::{Graph, Node};
    use crate::testing::{build_graph, edges, graph_pairs};

    #[test]
    fn chain_equals_the_combinators() {
        let built = Expr::new(Gnp::new(8, 0.4, 3))
            .complement()
            .union(edges(8, &[(0, 1), (2, 5)]))
            .difference(Expr::new(Complete::new(3)));
        let built = build_graph(built.term(), false);
        let direct = build_graph(
            Difference::new(
                Union::new(
                    Complement::new(Gnp::new(8, 0.4, 3)),
                    edges(8, &[(0, 1), (2, 5)]),
                ),
                Complete::new(3),
            ),
            false,
        );
        assert_eq!(built.order(), direct.order());
        assert!(built.edges().eq(direct.edges()));
    }

    #[test]
    fn every_combinator_is_available() {
        let a = || leaf(4, &[(0, 1), (1, 2)]);
        let b = || leaf(4, &[(1, 2), (2, 3)]);
        assert_eq!(built(&a().union(b())), [(0, 1), (1, 2), (2, 3)]);
        assert_eq!(built(&a().intersection(b())), [(1, 2)]);
        assert_eq!(built(&a().difference(b())), [(0, 1)]);
        assert_eq!(
            built(&a().disjoint_union(b())),
            [(0, 1), (1, 2), (5, 6), (6, 7)]
        );
        assert_eq!(built(&a().complement()), [(0, 2), (0, 3), (1, 3), (2, 3)]);
    }

    fn leaf(order: u64, pairs: &[(Node, Node)]) -> Expr {
        Expr::from(edges(order, pairs))
    }

    fn built(b: &Expr) -> Vec<(Node, Node)> {
        graph_pairs(&build_graph(b.term(), false))
    }

    #[test]
    fn reverse_turns_the_arcs_around() {
        let g = build_graph(leaf(3, &[(0, 1), (1, 2)]).reverse().term(), true);
        assert_eq!(graph_pairs(&g), [(1, 0), (2, 1)]);
    }

    #[test]
    fn each_operator_is_its_named_method() {
        let a = || leaf(4, &[(0, 1), (1, 2)]);
        let b = || leaf(3, &[(1, 2), (2, 0)]);
        assert_eq!(built(&(a() | b())), built(&a().union(b())));
        assert_eq!(built(&(a() & b())), built(&a().intersection(b())));
        assert_eq!(built(&(a() - b())), built(&a().difference(b())));
        assert_eq!(built(&(a() + b())), built(&a().disjoint_union(b())));
        assert_eq!(built(&(a() * b())), built(&a().cartesian_product(b())));
        assert_eq!(built(&!a()), built(&a().complement()));
    }

    // The point of this test is to leave the parentheses out.
    #[allow(clippy::precedence)]
    #[test]
    fn the_precedence_is_rusts_as_in_the_spec() {
        let a = || leaf(5, &[(0, 1), (1, 2), (3, 4)]);
        let b = || leaf(5, &[(1, 2), (2, 3)]);
        let c = || leaf(5, &[(0, 1), (3, 4)]);
        // `&` binds tighter than `|`.
        assert_eq!(
            built(&(a() | b() & c())),
            built(&a().union(b().intersection(c())))
        );
        // `-` binds tighter than `|` and `&`.
        assert_eq!(
            built(&(a() - b() | c())),
            built(&a().difference(b()).union(c()))
        );
        assert_eq!(
            built(&(a() & b() - c())),
            built(&a().intersection(b().difference(c())))
        );
        // `+` and `-` share a level, from the left.
        assert_eq!(
            built(&(a() + b() - c())),
            built(&a().disjoint_union(b()).difference(c()))
        );
        // `!` binds tightest.
        assert_eq!(built(&(!a() | b())), built(&a().complement().union(b())));
    }

    #[test]
    fn a_reference_works_on_either_side_and_leaves_the_builder_usable() {
        let a = leaf(4, &[(0, 1), (1, 2)]);
        let b = leaf(4, &[(1, 2), (2, 3)]);
        assert_eq!(built(&(&a | &b)), built(&a.clone().union(b.clone())));
        assert_eq!(built(&(a.clone() & &b)), [(1, 2)]);
        assert_eq!(built(&(&a - b.clone())), [(0, 1)]);
        assert_eq!(built(&!&a), built(&a.clone().complement()));
        // `a` and `b` are still there.
        assert_eq!(built(&a), [(0, 1), (1, 2)]);
        assert_eq!(built(&b), [(1, 2), (2, 3)]);
    }

    #[test]
    fn an_expression_is_a_right_operand() {
        let g = Expr::new(Complete::new(3)) | Complete::new(5);
        assert_eq!(build_graph(g.term(), false).order(), 5);
        let g = Expr::new(Complete::new(3)) & Gnp::new(3, 1.0, 1);
        assert_eq!(build_graph(g.term(), false).size(), 3);
    }

    #[test]
    fn with_min_order_raises_the_order_and_never_lowers_it() {
        let g = build_graph(Expr::new(Complete::new(3)).with_min_order(10).term(), false);
        assert_eq!((g.order(), g.size()), (10, 3));
        let g = build_graph(Expr::new(Complete::new(3)).with_min_order(2).term(), false);
        assert_eq!((g.order(), g.size()), (3, 3));
    }
}
