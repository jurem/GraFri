//! [`Gnp`]: the random graph G(n, p), decided pair by pair.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Error, Errors, Node, possible_edges};
use crate::random::{self, Key};

/// The random graph G(n, p): every possible edge is present independently with probability `p`.
/// The possible edges are the pairs `u < v` if undirected, and all ordered pairs `u != v` if
/// directed; no self-loops.
///
/// The graph is a function of the order, `p`, the `stream` and the seed of the context: the same
/// values always give the same graph, and another stream or seed gives an independent sample.
/// Each pair is decided on its own, so the undirected graph is a subgraph of the directed one
/// with the same parameters and context.
#[derive(Clone)]
pub struct Gnp {
    order: u64,
    p: f64,
    stream: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Gnp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Gnp")
            .field(&self.order)
            .field(&self.p)
            .field(&self.stream)
            .finish()
    }
}

impl Gnp {
    /// G(`order`, `p`) on the random stream number `stream`.
    pub fn new(order: u64, p: f64, stream: u64) -> Gnp {
        Gnp { order, p, stream }
    }

    /// The seed that the pairs are hashed with under `ctx`.
    fn seed(&self, ctx: &Context) -> u64 {
        random::stream_seed(ctx.seed, self.stream)
    }
}

/// Whether the possible edge `edge` is present: `unit(edge.fold(seed)) < p`, where `seed` is
/// `stream_seed(ctx.seed, stream)`.
fn is_present(seed: u64, p: f64, edge: Edge) -> bool {
    random::unit((edge.u, edge.v).fold(seed)) < p
}

impl Term for Gnp {
    fn order(&self, _ctx: &Context) -> u64 {
        self.order
    }

    /// Filters the possible pairs, in O(order²) time and O(1) memory.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let (seed, p) = (self.seed(ctx), self.p);
        Box::new(possible_edges(self.order, ctx.directed).filter(move |&e| is_present(seed, p, e)))
    }

    /// O(1).
    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let edge = Edge::new(ctx.directed, u, v);
        let possible = edge.u != edge.v && self.has_node(ctx, edge.u) && self.has_node(ctx, edge.v);
        possible && is_present(self.seed(ctx), self.p, edge)
    }

    /// The order must be at most [`MAX_ORDER`](crate::MAX_ORDER), and `p` in `[0, 1]`.
    fn validate(&self) -> Result<(), Errors> {
        let p = if (0.0..=1.0).contains(&self.p) {
            Ok(())
        } else {
            Err(Error::InvalidParam {
                name: "p",
                message: format!("must be a probability in [0, 1], got {}", self.p),
            }
            .into())
        };
        Errors::collect([check_order("order", u128::from(self.order)), p])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::Graph;
    use crate::graphs::computed::GraphOf;
    use crate::graphs::computed::generators::Complete;
    use crate::testing::{build_graph, build_graph_in, check_consistent};

    fn ctx(directed: bool, seed: u64) -> Context {
        Context::new(directed).with_seed(seed)
    }

    fn gnp(order: u64, p: f64, stream: u64, ctx: Context) -> GraphOf {
        build_graph_in(Gnp::new(order, p, stream), ctx)
    }

    #[test]
    fn edges_follow_the_reference_vector() {
        // Computed by an independent implementation of `mix`, `unit` and
        // `stream_seed` (n = 6, p = 0.5, master seed 3, stream 2).
        let g = Gnp::new(6, 0.5, 2);
        let seeded = |directed| {
            let c = ctx(directed, 3);
            g.edges(&c).map(|e| (e.u, e.v)).collect::<Vec<_>>()
        };
        assert_eq!(seeded(false), [(0, 2), (0, 5), (1, 2), (1, 3), (1, 4)]);
        assert_eq!(
            seeded(true),
            [
                (0, 2),
                (0, 5),
                (1, 0),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 1),
                (3, 0),
                (4, 2),
                (4, 3),
                (5, 0),
                (5, 1),
                (5, 3)
            ]
        );
    }

    #[test]
    fn p_zero_is_empty_and_p_one_is_complete() {
        for directed in [false, true] {
            let none = gnp(6, 0.0, 1, ctx(directed, 0));
            assert_eq!((none.order(), none.size()), (6, 0));
            let all = gnp(6, 1.0, 1, ctx(directed, 0));
            let complete = build_graph(Complete::new(6), directed);
            assert!(all.edges().eq(complete.edges()));
        }
    }

    #[test]
    fn check_rejects_invalid_probability() {
        for p in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
            let errors = Gnp::new(5, p, 1).validate().unwrap_err();
            assert!(matches!(
                errors.as_slice(),
                [Error::InvalidParam { name: "p", .. }]
            ));
        }
    }

    #[test]
    fn same_context_same_graph_other_seed_or_stream_other_graph() {
        let edges = |stream, seed| {
            gnp(30, 0.3, stream, ctx(false, seed))
                .edges()
                .collect::<Vec<_>>()
        };
        let a = edges(5, 1);
        assert_eq!(a, edges(5, 1));
        assert_ne!(a, edges(6, 1));
        assert_ne!(a, edges(5, 2));
    }

    #[test]
    fn edge_count_is_near_its_expectation() {
        // 4950 pairs, mean 2475, standard deviation about 35.
        let g = gnp(100, 0.5, 0, ctx(false, 11));
        assert!((2300..2650).contains(&g.size()));
    }

    #[test]
    fn has_edge_and_size_match_edges_under_both_directed_and_undirected() {
        check_consistent(&Gnp::new(8, 0.5, 2), 10);
    }

    #[test]
    fn the_undirected_graph_is_a_subgraph_of_the_directed_one() {
        let g = Gnp::new(30, 0.3, 4);
        let directed = g.edges(&ctx(true, 9)).collect::<Vec<_>>();
        for e in g.edges(&ctx(false, 9)) {
            assert!(directed.contains(&e), "{e:?}");
        }
    }
}
