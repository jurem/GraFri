//! [`DisjointUnion`]: two terms side by side.

use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The disjoint union of two graphs: the nodes of `a` keep their numbers, the nodes of `b` are
/// shifted up by `a.order()`, and no edge joins the two. Self-loops are kept.
#[derive(Debug, Clone)]
pub struct DisjointUnion(Arc<dyn Term>, Arc<dyn Term>);

impl DisjointUnion {
    /// The disjoint union of `a` and `b`.
    pub fn new(a: impl Term + 'static, b: impl Term + 'static) -> DisjointUnion {
        DisjointUnion(a.shared(), b.shared())
    }
}

impl Term for DisjointUnion {
    fn order(&self, ctx: &Context) -> u64 {
        self.0.order(ctx) + self.1.order(ctx)
    }

    fn size(&self, ctx: &Context) -> u64 {
        self.0.size(ctx) + self.1.size(ctx)
    }

    /// Chains the operands' edges: the second is shifted past the first, so they stay sorted.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let offset = self.0.order(ctx);
        let shifted = self.1.edges(ctx).map(move |e| Edge {
            u: e.u + offset,
            v: e.v + offset,
        });
        Box::new(self.0.edges(ctx).chain(shifted))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let offset = self.0.order(ctx);
        if u < offset && v < offset {
            self.0.has_edge(ctx, u, v)
        } else if u >= offset && v >= offset {
            self.1.has_edge(ctx, u - offset, v - offset)
        } else {
            false
        }
    }

    fn validate(&self) -> Result<(), Errors> {
        Errors::collect([self.0.validate(), self.1.validate()])
    }

    /// The two orders must add up to at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn resolve(&self, ctx: &Context) -> Result<(), Errors> {
        // The orders are read only after the operands passed: they may be out of range before.
        Errors::collect([self.0.resolve(ctx), self.1.resolve(ctx)])?;
        let orders = u128::from(self.0.order(ctx)) + u128::from(self.1.order(ctx));
        check_order("orders", orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::Error;
    use crate::graphs::computed::MAX_ORDER;
    use crate::graphs::computed::generators::Complete;
    use crate::testing::{check_consistent, edges, pairs_of};

    #[test]
    fn shifts_the_second_operand() {
        let a = edges(3, &[(0, 1), (1, 2)]);
        let b = edges(2, &[(0, 1)]);
        let u = DisjointUnion::new(a, b);
        assert_eq!(pairs_of(&u, false), [(0, 1), (1, 2), (3, 4)]);
        let ctx = Context::undirected();
        assert_eq!((u.order(&ctx), u.size(&ctx)), (5, 3));
    }

    #[test]
    fn directed_and_loops_are_kept() {
        let a = edges(2, &[(1, 0), (0, 0)]);
        let b = edges(2, &[(1, 0)]);
        let u = DisjointUnion::new(a, b);
        assert_eq!(pairs_of(&u, true), [(0, 0), (1, 0), (3, 2)]);
    }

    #[test]
    fn empty_operands() {
        let ctx = Context::undirected();
        let u = DisjointUnion::new(Complete::new(0), Complete::new(3));
        assert_eq!(pairs_of(&u, false), [(0, 1), (0, 2), (1, 2)]);
        let u = DisjointUnion::new(Complete::new(3), Complete::new(0));
        assert_eq!((u.order(&ctx), u.size(&ctx)), (3, 3));
    }

    #[test]
    fn has_edge_and_size_match_edges() {
        let a = edges(3, &[(0, 1), (2, 0)]);
        check_consistent(&DisjointUnion::new(a, Complete::new(3)), 7);
    }

    #[test]
    fn edges_are_in_canonical_order_and_nest() {
        let ctx = Context::undirected();
        let inner = DisjointUnion::new(Complete::new(3), Complete::new(2));
        let u = DisjointUnion::new(inner, Complete::new(3));
        let edges: Vec<_> = u.edges(&ctx).collect();
        assert!(edges.windows(2).all(|w| w[0] < w[1]));
        assert_eq!((u.order(&ctx), u.size(&ctx)), (8, 3 + 1 + 3));
    }

    #[test]
    fn check_requires_the_orders_to_add_up() {
        let ctx = Context::undirected();
        let fine = DisjointUnion::new(Complete::new(3), Complete::new(4));
        assert_eq!(fine.check(&ctx), Ok(()));
        // Orders that add up to exactly the maximum are accepted.
        let at_the_limit = DisjointUnion::new(edges(MAX_ORDER - 2, &[]), edges(2, &[]));
        assert_eq!(at_the_limit.check(&ctx), Ok(()));
        let overflow = DisjointUnion::new(edges(MAX_ORDER, &[]), edges(1, &[]));
        assert_eq!(overflow.validate(), Ok(()));
        let errors = overflow.check(&ctx).unwrap_err();
        assert!(matches!(
            errors.as_slice(),
            [Error::InvalidParam { name: "orders", .. }]
        ));
        // The errors of the operands stop `check` before the one about the orders.
        let both = DisjointUnion::new(edges(MAX_ORDER, &[(0, u64::MAX)]), edges(2, &[(0, 5)]));
        assert_eq!(both.check(&ctx).unwrap_err().as_slice().len(), 2);
        assert_eq!(both.resolve(&ctx).unwrap_err().as_slice().len(), 1);
    }
}
