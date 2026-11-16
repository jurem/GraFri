//! [`Intersection`]: the intersection of two terms on shared nodes.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::Term;
use crate::graphs::{Edge, Errors, Node};

/// The intersection of two graphs: nodes are identified by number, and an edge is present if it is
/// in both `a` and `b`. The order is the smaller of the two.
#[derive(Debug, Clone)]
pub struct Intersection(Arc<dyn Term>, Arc<dyn Term>);

impl Intersection {
    /// The intersection of `a` and `b`.
    pub fn new(a: impl Term + 'static, b: impl Term + 'static) -> Intersection {
        Intersection(a.shared(), b.shared())
    }
}

impl Term for Intersection {
    fn order(&self, ctx: &Context) -> u64 {
        self.0.order(ctx).min(self.1.order(ctx))
    }

    /// Merges the operands' sorted edges, in O(a.size() + b.size()).
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let mut a = self.0.edges(ctx).peekable();
        let mut b = self.1.edges(ctx).peekable();
        Box::new(std::iter::from_fn(move || {
            loop {
                let x = a.peek().copied()?;
                let y = b.peek().copied()?;
                match x.cmp(&y) {
                    Ordering::Less => {
                        a.next();
                    }
                    Ordering::Greater => {
                        b.next();
                    }
                    Ordering::Equal => {
                        b.next();
                        return a.next();
                    }
                }
            }
        }))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        self.0.has_edge(ctx, u, v) && self.1.has_edge(ctx, u, v)
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
        check_set_operation(Intersection::new, |a, b| {
            a.intersection(b).copied().collect()
        });
    }

    #[test]
    fn order_is_the_smaller_one() {
        let ctx = Context::undirected();
        let a = edges(3, &[(0, 1), (1, 2)]).shared();
        let b = edges(6, &[(0, 1), (4, 5)]).shared();
        let i = Intersection::new(Arc::clone(&a), Arc::clone(&b));
        assert_eq!((i.order(&ctx), pairs_of(&i, false)), (3, vec![(0, 1)]));
        let i = Intersection::new(b, a);
        assert_eq!((i.order(&ctx), pairs_of(&i, false)), (3, vec![(0, 1)]));
    }
}
