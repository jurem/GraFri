//! The [`Term`] trait.

use std::fmt;
use std::sync::Arc;

use crate::Context;
use crate::graphs::{Edge, Errors, Node};

/// A graph term: a tree of generators and combinators with no directedness of its own. Every
/// method takes the [`Context`] that supplies it.
pub trait Term: fmt::Debug + Send + Sync {
    /// The number of nodes. It takes the context so that an order may depend on the seed; none does
    /// yet. Until the term is checked, the order of a combinator may exceed
    /// [`MAX_ORDER`](crate::MAX_ORDER).
    fn order(&self, ctx: &Context) -> u64;

    /// The nodes, in ascending order.
    fn nodes(&self, ctx: &Context) -> std::ops::Range<Node> {
        0..self.order(ctx)
    }

    /// Whether `v` is a node of the graph.
    fn has_node(&self, ctx: &Context, v: Node) -> bool {
        v < self.order(ctx)
    }

    /// The number of edges. O(size) by default.
    fn size(&self, ctx: &Context) -> u64 {
        self.edges(ctx).count() as u64
    }

    /// The edges under `ctx`, as [`Graph::edges`](crate::graphs::Graph::edges) yields them.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_>;

    /// Whether the graph has the edge `u -> v` (`u -- v` if undirected). O(size) by default, which
    /// relies on `edges` being sorted.
    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let probe = Edge::new(ctx.directed, u, v);
        self.edges(ctx).find(|e| *e >= probe) == Some(probe)
    }

    /// Reports every problem of the term and its operands that can be found without a
    /// context: parameters out of range, and an order above [`MAX_ORDER`](crate::MAX_ORDER). The
    /// default accepts everything.
    fn validate(&self) -> Result<(), Errors> {
        Ok(())
    }

    /// Reports every problem that needs the context, such as a combinator whose order, a sum or a
    /// product of its operands' orders, is above [`MAX_ORDER`](crate::MAX_ORDER). It assumes
    /// [`validate`](Term::validate) passed. The default accepts everything.
    fn resolve(&self, _ctx: &Context) -> Result<(), Errors> {
        Ok(())
    }

    /// [`validate`](Term::validate), then [`resolve`](Term::resolve) if it found nothing, so it
    /// reports every problem of the first pass that fails. Reading a term that was not
    /// checked is a misuse: the results are unspecified and it may panic.
    fn check(&self, ctx: &Context) -> Result<(), Errors> {
        self.validate()?;
        self.resolve(ctx)
    }

    /// The term in an `Arc`.
    fn shared(self) -> Arc<dyn Term>
    where
        Self: Sized + 'static,
    {
        Arc::new(self)
    }
}

/// The methods of [`Term`] that a pointer forwards.
macro_rules! forward_methods {
    () => {
        fn order(&self, ctx: &Context) -> u64 {
            (**self).order(ctx)
        }

        fn nodes(&self, ctx: &Context) -> std::ops::Range<Node> {
            (**self).nodes(ctx)
        }

        fn has_node(&self, ctx: &Context, v: Node) -> bool {
            (**self).has_node(ctx, v)
        }

        fn size(&self, ctx: &Context) -> u64 {
            (**self).size(ctx)
        }

        fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
            (**self).edges(ctx)
        }

        fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
            (**self).has_edge(ctx, u, v)
        }

        fn validate(&self) -> Result<(), Errors> {
            (**self).validate()
        }

        fn resolve(&self, ctx: &Context) -> Result<(), Errors> {
            (**self).resolve(ctx)
        }

        fn check(&self, ctx: &Context) -> Result<(), Errors> {
            (**self).check(ctx)
        }
    };
}

/// Forwards [`Term`] through pointer types.
macro_rules! forward_expr {
    ($($ptr:ty),*) => {$(
        impl<E: Term + ?Sized> Term for $ptr {
            forward_methods!();
        }
    )*};
}

forward_expr!(&E, Box<E>);

/// A shared term; [`Term::shared`] returns the same `Arc`.
impl<E: Term + 'static> Term for Arc<E> {
    forward_methods!();

    fn shared(self) -> Arc<dyn Term> {
        self
    }
}

/// The same for a type-erased `Arc`.
impl Term for Arc<dyn Term> {
    forward_methods!();

    fn shared(self) -> Arc<dyn Term> {
        self
    }
}
