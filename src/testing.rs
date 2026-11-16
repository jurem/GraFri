//! Helpers for the unit tests.

use std::collections::BTreeSet;

use crate::graphs::{Edge, Error, Graph, Node};
use crate::labels::{self, Labeling};

use crate::Context;
use crate::graphs::computed::generators::Edges;
use crate::graphs::computed::{GraphOf, MAX_ORDER, Term};

/// The term as an undirected or directed graph under the default context, unchecked.
pub(crate) fn build_graph(term: impl Term + 'static, directed: bool) -> GraphOf {
    build_graph_in(term, Context::new(directed))
}

/// The same under `ctx`, for a seed or a shared context.
pub(crate) fn build_graph_in(term: impl Term + 'static, ctx: impl Into<Context>) -> GraphOf {
    GraphOf::new(term.shared(), ctx)
}

/// The graph with `order` nodes and the given pairs, unchecked.
pub(crate) fn edges(order: u64, pairs: &[(Node, Node)]) -> Edges {
    Edges::new(order, pairs.iter().copied())
}

/// The edges of `e` under the given directedness, as pairs.
pub(crate) fn pairs_of(e: &impl Term, directed: bool) -> Vec<(Node, Node)> {
    e.edges(&Context::new(directed))
        .map(|e| (e.u, e.v))
        .collect()
}

/// The edges of `g` as pairs.
pub(crate) fn graph_pairs(g: &impl Graph) -> Vec<(Node, Node)> {
    g.edges().map(|e| (e.u, e.v)).collect()
}

/// Every graph on `order` nodes, loops included, as its sorted pairs: undirected `u <= v`.
pub(crate) fn all_graphs(order: u64, directed: bool) -> Vec<Vec<(Node, Node)>> {
    let possible: Vec<(Node, Node)> = (0..order)
        .flat_map(|u| (0..order).map(move |v| (u, v)))
        .filter(|&(u, v)| directed || u <= v)
        .collect();
    (0..1u32 << possible.len())
        .map(|mask| {
            let chosen = possible.iter().enumerate();
            chosen
                .filter(|&(i, _)| mask >> i & 1 == 1)
                .map(|(_, &pair)| pair)
                .collect()
        })
        .collect()
}

/// Checks the combinator `make` against `expected`, an operation on the sets of edges, over all
/// pairs of graphs on 3 nodes if undirected and on 2 nodes if directed.
pub(crate) fn check_set_operation<E: Term>(
    make: impl Fn(Edges, Edges) -> E,
    expected: impl Fn(&BTreeSet<(Node, Node)>, &BTreeSet<(Node, Node)>) -> Vec<(Node, Node)>,
) {
    for (order, directed) in [(3, false), (2, true)] {
        let graphs = all_graphs(order, directed);
        for a in &graphs {
            for b in &graphs {
                let (set_a, set_b) = (a.iter().copied().collect(), b.iter().copied().collect());
                let e = make(edges(order, a), edges(order, b));
                assert_eq!(
                    pairs_of(&e, directed),
                    expected(&set_a, &set_b),
                    "{a:?} and {b:?} directed={directed}"
                );
                check_consistent(&e, order + 1);
            }
        }
    }
}

/// Checks that `make(order)` is valid for the largest order and invalid for one more, with a
/// single parameter error.
pub(crate) fn check_order_limit<E: Term>(make: impl Fn(u64) -> E) {
    assert_eq!(make(MAX_ORDER).validate(), Ok(()));
    let errors = make(MAX_ORDER + 1).validate().unwrap_err();
    assert!(matches!(errors.as_slice(), [Error::InvalidParam { .. }]));
}

/// Checks that `edges` is strictly increasing, and that `has_edge` and `size` agree with `edges`,
/// for both directednesses, over all pairs of nodes up to `limit`. It also probes pairs near
/// `u64::MAX`, which are never edges.
pub(crate) fn check_consistent(e: &impl Term, limit: u64) {
    for directed in [false, true] {
        let ctx = Context::new(directed);
        let edges: Vec<Edge> = e.edges(&ctx).collect();
        assert!(
            edges.windows(2).all(|w| w[0] < w[1]),
            "edges are not strictly increasing (directed={directed}): {edges:?}"
        );
        assert_eq!(e.size(&ctx), edges.len() as u64);
        // Nodes far outside the graph are never joined, and asking does not overflow.
        for (s, t) in [(u64::MAX, u64::MAX), (0, u64::MAX), (u64::MAX, 0)] {
            assert!(!e.has_edge(&ctx, s, t), "directed={directed} ({s},{t})");
        }
        for s in 0..limit {
            for t in 0..limit {
                let probe = Edge::new(directed, s, t);
                assert_eq!(
                    e.has_edge(&ctx, s, t),
                    e.edges(&ctx).any(|x| x == probe),
                    "directed={directed} ({s},{t})"
                );
            }
        }
    }
}

#[cfg(test)]
mod check_consistent_tests {
    use super::*;

    /// A broken term: its edges are not in ascending order.
    #[derive(Debug)]
    struct Unsorted(Vec<Edge>);

    impl Term for Unsorted {
        fn order(&self, _ctx: &Context) -> u64 {
            4
        }

        fn edges(&self, _ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
            Box::new(self.0.iter().copied())
        }
    }

    #[test]
    fn accepts_sorted_edges() {
        check_consistent(
            &Unsorted(vec![Edge::directed(0, 1), Edge::directed(1, 2)]),
            4,
        );
    }

    #[test]
    #[should_panic(expected = "not strictly increasing")]
    fn rejects_edges_out_of_order() {
        check_consistent(
            &Unsorted(vec![Edge::directed(1, 2), Edge::directed(0, 1)]),
            4,
        );
    }

    #[test]
    #[should_panic(expected = "not strictly increasing")]
    fn rejects_repeated_edges() {
        check_consistent(
            &Unsorted(vec![Edge::directed(0, 1), Edge::directed(0, 1)]),
            4,
        );
    }
}

#[cfg(test)]
mod thread_safety {
    use super::build_graph_in;
    use crate::*;

    fn is_send_and_sync<T: Send + Sync>() {}

    #[test]
    fn the_main_types_can_be_shared_between_threads() {
        is_send_and_sync::<Expr>();
        is_send_and_sync::<Spec>();
        is_send_and_sync::<GraphOf>();
        is_send_and_sync::<Context>();
        is_send_and_sync::<EdgeList>();
        is_send_and_sync::<Labeled<GraphOf>>();
        is_send_and_sync::<Labeled<EdgeList>>();
        is_send_and_sync::<Box<dyn Term>>();
        is_send_and_sync::<std::sync::Arc<dyn Term>>();
        is_send_and_sync::<std::sync::Arc<dyn NodeLabeling>>();
        is_send_and_sync::<std::sync::Arc<dyn EdgeLabeling>>();
        is_send_and_sync::<DenseLabeling>();
        is_send_and_sync::<KeyedLabeling<Edge>>();
        is_send_and_sync::<IntLabeling>();
        is_send_and_sync::<ChoiceLabeling>();
    }

    #[test]
    fn threads_reading_one_graph_see_the_same_edges() {
        // `RandomRecursiveTree` caches its sorted edges in the graph's context, behind a lock,
        // so this also exercises that cache from several threads at once.
        let graph = build_graph_in(
            RandomRecursiveTree::new(60, 1),
            Context::undirected().with_seed(3),
        );
        let expected: Vec<Edge> = graph.edges().collect();
        let labels = Labeled::new(graph.clone())
            .with_edge_labeling(IntLabeling::new("w", 1..100))
            .unwrap();
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    assert_eq!(graph.edges().collect::<Vec<_>>(), expected);
                    for e in &expected {
                        assert!(labels.edge_label("w", e.u, e.v).is_some());
                    }
                });
            }
        });
    }
}

/// A context with the given master seed.
pub(crate) fn context_at(seed: u64) -> Context {
    Context::default().with_seed(seed)
}

/// Every problem that `validate` finds in a labeling, in order (none if it is valid).
pub(crate) fn label_problems(l: &impl Labeling<u64>) -> Vec<labels::Error> {
    l.validate()
        .err()
        .map(|errors| errors.as_slice().to_vec())
        .unwrap_or_default()
}
