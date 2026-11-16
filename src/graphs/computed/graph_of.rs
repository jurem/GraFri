//! [`GraphOf`]: a term with its context fixed, which is a graph.

use std::fmt;
use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::Term;
use crate::graphs::{Edge, Graph, Node};

/// A term together with the [`Context`] that fixes it, which makes it a [`Graph`]: every
/// query is forwarded to the term. Meant for the root of a term only.
#[derive(Clone)]
pub struct GraphOf {
    term: Arc<dyn Term>,
    ctx: Context,
}

impl GraphOf {
    /// The graph that `term` describes under `ctx`, with a cache of its own that its clones share.
    ///
    /// Does not call [`Term::check`]; [`Spec::build`](crate::Spec::build) does. `term` must have
    /// passed it for `ctx`: reading a term that did not may panic or give wrong results.
    pub fn new(term: Arc<dyn Term>, ctx: impl Into<Context>) -> GraphOf {
        GraphOf {
            term,
            ctx: ctx.into().with_fresh_cache(),
        }
    }

    /// The context the term is read under.
    pub fn context(&self) -> &Context {
        &self.ctx
    }
}

/// Shown as the term, the directedness and the seed. The cache is left out.
impl fmt::Debug for GraphOf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let directedness = if self.ctx.directed {
            "directed"
        } else {
            "undirected"
        };
        f.debug_tuple("GraphOf")
            .field(&self.term)
            .field(&format_args!("{directedness}"))
            .field(&format_args!("seed {}", self.ctx.seed))
            .finish()
    }
}

impl Graph for GraphOf {
    fn order(&self) -> u64 {
        self.term.order(&self.ctx)
    }

    fn nodes(&self) -> std::ops::Range<Node> {
        self.term.nodes(&self.ctx)
    }

    fn has_node(&self, v: Node) -> bool {
        self.term.has_node(&self.ctx, v)
    }

    fn size(&self) -> u64 {
        self.term.size(&self.ctx)
    }

    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        self.term.edges(&self.ctx)
    }

    fn has_edge(&self, u: Node, v: Node) -> bool {
        self.term.has_edge(&self.ctx, u, v)
    }

    fn is_directed(&self) -> bool {
        self.ctx.directed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{build_graph, build_graph_in};

    /// A path 0 - 1 - 2, as arcs 0 -> 1 -> 2 if directed.
    #[derive(Debug)]
    struct Path3;

    impl Term for Path3 {
        fn order(&self, _ctx: &Context) -> u64 {
            3
        }

        fn edges(&self, _ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
            Box::new([Edge::directed(0, 1), Edge::directed(1, 2)].into_iter())
        }
    }

    #[test]
    fn forwards_queries_with_its_context() {
        let g = build_graph(Path3, true);
        assert_eq!((g.order(), g.size()), (3, 2));
        assert_eq!(g.nodes(), 0..3);
        assert!(g.is_directed());
        assert!(g.has_edge(0, 1));
        assert!(!g.has_edge(1, 0));
        assert_eq!(g.edges().count(), 2);
    }

    #[test]
    fn forwards_the_undirected_context_too() {
        let g = build_graph(Path3, false);
        assert!(!g.is_directed());
        assert!(g.has_edge(0, 1) && g.has_edge(1, 0));
    }

    #[test]
    fn each_graph_has_a_cache_of_its_own_and_clones_share_it() {
        let ctx = Context::undirected();
        let edges = || vec![Edge::undirected(0, 1)];
        let a = build_graph_in(Path3, &ctx);
        let b = build_graph_in(Path3, &ctx);
        let first = a.context().cached("leaf", [0; 3], edges);
        // A clone of the graph shares the entry; another graph made from the same context does not.
        let again = a
            .clone()
            .context()
            .cached("leaf", [0; 3], || unreachable!());
        assert!(Arc::ptr_eq(&first, &again));
        assert!(!Arc::ptr_eq(
            &first,
            &b.context().cached("leaf", [0; 3], edges)
        ));
        assert!(!Arc::ptr_eq(&first, &ctx.cached("leaf", [0; 3], edges)));
    }
}
