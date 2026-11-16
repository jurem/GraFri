//! [`Difference`]: the edges of one term that are not in another.

use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::Term;
use crate::graphs::{Edge, Errors, Node};

/// The difference of two graphs: nodes are identified by number, and an edge is present if it is in
/// `a` but not in `b`. The order is that of `a`.
#[derive(Debug, Clone)]
pub struct Difference(Arc<dyn Term>, Arc<dyn Term>);

impl Difference {
    /// The edges of `a` that are not in `b`.
    pub fn new(a: impl Term + 'static, b: impl Term + 'static) -> Difference {
        Difference(a.shared(), b.shared())
    }
}

impl Term for Difference {
    fn order(&self, ctx: &Context) -> u64 {
        self.0.order(ctx)
    }

    /// Merges the operands' sorted edges, in O(a.size() + b.size()).
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let mut b = self.1.edges(ctx).peekable();
        Box::new(self.0.edges(ctx).filter(move |x| {
            while b.next_if(|y| y < x).is_some() {}
            b.next_if_eq(x).is_none()
        }))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        self.0.has_edge(ctx, u, v) && !self.1.has_edge(ctx, u, v)
    }

    fn validate(&self) -> Result<(), Errors> {
        Errors::collect([self.0.validate(), self.1.validate()])
    }

    fn resolve(&self, ctx: &Context) -> Result<(), Errors> {
        Errors::collect([self.0.resolve(ctx), self.1.resolve(ctx)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{check_set_operation, edges, pairs_of};

    #[test]
    fn matches_the_definition() {
        check_set_operation(Difference::new, |a, b| a.difference(b).copied().collect());
    }

    #[test]
    fn order_is_that_of_the_first_operand() {
        let ctx = Context::undirected();
        let a = edges(3, &[(0, 1), (1, 2)]).shared();
        let b = edges(6, &[(0, 1), (4, 5)]).shared();
        let d = Difference::new(Arc::clone(&a), Arc::clone(&b));
        assert_eq!((d.order(&ctx), pairs_of(&d, false)), (3, vec![(1, 2)]));
        let d = Difference::new(b, a);
        assert_eq!((d.order(&ctx), pairs_of(&d, false)), (6, vec![(4, 5)]));
    }
}
