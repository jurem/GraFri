//! [`Union`]: the union of two terms on shared nodes.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::Term;
use crate::graphs::{Edge, Errors, Node};

/// The union of two graphs: nodes are identified by number, and an edge is present if it is in `a`
/// or in `b`. The order is the larger of the two.
#[derive(Debug, Clone)]
pub struct Union(Arc<dyn Term>, Arc<dyn Term>);

impl Union {
    /// The union of `a` and `b`.
    pub fn new(a: impl Term + 'static, b: impl Term + 'static) -> Union {
        Union(a.shared(), b.shared())
    }
}

impl Term for Union {
    fn order(&self, ctx: &Context) -> u64 {
        self.0.order(ctx).max(self.1.order(ctx))
    }

    /// Merges the operands' sorted edges and drops duplicates, in O(a.size() + b.size()).
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let mut a = self.0.edges(ctx).peekable();
        let mut b = self.1.edges(ctx).peekable();
        Box::new(std::iter::from_fn(move || {
            match (a.peek().copied(), b.peek().copied()) {
                (None, None) => None,
                (Some(_), None) => a.next(),
                (None, Some(_)) => b.next(),
                (Some(x), Some(y)) => match x.cmp(&y) {
                    Ordering::Less => a.next(),
                    Ordering::Greater => b.next(),
                    Ordering::Equal => {
                        b.next();
                        a.next()
                    }
                },
            }
        }))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        self.0.has_edge(ctx, u, v) || self.1.has_edge(ctx, u, v)
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
    use crate::graphs::Error;
    use crate::graphs::computed::combinators::Complement;
    use crate::graphs::computed::generators::{Complete, Edges, Gnp};
    use crate::testing::{check_set_operation, edges, pairs_of};

    #[test]
    fn validate_reports_the_errors_of_both_operands() {
        assert_eq!(
            Union::new(Complete::new(3), Complete::new(4)).validate(),
            Ok(())
        );
        let bad = Union::new(
            Edges::new(2, [(0, 5)]),
            Complement::new(Gnp::new(3, 2.0, 1)),
        );
        let errors = bad.validate().unwrap_err();
        assert!(matches!(
            errors.as_slice(),
            [
                Error::EdgeOutOfRange { .. },
                Error::InvalidParam { name: "p", .. }
            ]
        ));
    }

    #[test]
    fn matches_the_definition() {
        check_set_operation(Union::new, |a, b| a.union(b).copied().collect());
    }

    #[test]
    fn order_is_the_larger_one() {
        let ctx = Context::undirected();
        let a = edges(3, &[(0, 1)]).shared();
        let b = edges(6, &[(4, 5)]).shared();
        let u = Union::new(Arc::clone(&a), Arc::clone(&b));
        assert_eq!(
            (u.order(&ctx), pairs_of(&u, false)),
            (6, vec![(0, 1), (4, 5)])
        );
        let u = Union::new(b, a);
        assert_eq!(
            (u.order(&ctx), pairs_of(&u, false)),
            (6, vec![(0, 1), (4, 5)])
        );
    }
}
