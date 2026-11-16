//! [`Hypercube`]: the hypercube graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{MAX_ORDER, Term};
use crate::graphs::{Edge, Error, Errors, Node};

/// The largest dimension whose order, `2^dimension`, is at most [`MAX_ORDER`].
const MAX_DIMENSION: u64 = 31;

/// The hypercube of the given dimension: `2^dimension` nodes, the numbers `0..2^dimension` read as
/// bit strings, joined if they differ in exactly one bit. Directed, every edge is both arcs. No
/// self-loops.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Hypercube {
    dimension: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Hypercube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Hypercube").field(&self.dimension).finish()
    }
}

impl Hypercube {
    /// The hypercube of the given dimension.
    pub fn new(dimension: u64) -> Hypercube {
        Hypercube { dimension }
    }
}

impl Term for Hypercube {
    fn order(&self, _ctx: &Context) -> u64 {
        1 << self.dimension
    }

    fn size(&self, ctx: &Context) -> u64 {
        let d = self.dimension;
        let undirected = if d == 0 { 0 } else { d << (d - 1) };
        if ctx.directed {
            2 * undirected
        } else {
            undirected
        }
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let (bits, directed) = (self.dimension, ctx.directed);
        // The neighbors of `u` are `u` with one bit flipped, in ascending order
        // (the flipped ones that were set are smaller than `u`, the others larger).
        // Undirected, only the larger ones are edges.
        Box::new((0..1u64 << bits).flat_map(move |u| {
            let mut neighbors: Vec<u64> = (0..bits)
                .map(|b| u ^ (1 << b))
                .filter(|&v| directed || u < v)
                .collect();
            neighbors.sort_unstable();
            neighbors.into_iter().map(move |v| Edge::directed(u, v))
        }))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        self.has_node(ctx, u) && self.has_node(ctx, v) && (u ^ v).is_power_of_two()
    }

    /// The order, `2^dimension`, must be at most [`MAX_ORDER`], which is a dimension of at most 31.
    fn validate(&self) -> Result<(), Errors> {
        if self.dimension <= MAX_DIMENSION {
            return Ok(());
        }
        Err(Error::InvalidParam {
            name: "dimension",
            message: format!(
                "the order 2^dimension must be at most {MAX_ORDER}, so dimension at most \
                 {MAX_DIMENSION}, got {}",
                self.dimension
            ),
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::Graph;
    use crate::testing::{build_graph, check_consistent, graph_pairs};

    #[test]
    fn the_cube_undirected() {
        let g = build_graph(Hypercube::new(3), false);
        assert_eq!(
            graph_pairs(&g),
            [
                (0, 1),
                (0, 2),
                (0, 4),
                (1, 3),
                (1, 5),
                (2, 3),
                (2, 6),
                (3, 7),
                (4, 5),
                (4, 6),
                (5, 7),
                (6, 7)
            ]
        );
        assert_eq!((g.order(), g.size()), (8, 12));
    }

    #[test]
    fn directed_has_both_arcs_of_every_edge_in_order() {
        let g = build_graph(Hypercube::new(2), true);
        assert_eq!(
            graph_pairs(&g),
            [
                (0, 1),
                (0, 2),
                (1, 0),
                (1, 3),
                (2, 0),
                (2, 3),
                (3, 1),
                (3, 2)
            ]
        );
        assert_eq!(g.size(), 8);
    }

    #[test]
    fn matches_the_definition_for_small_dimensions() {
        for d in 0..6u64 {
            let h = Hypercube::new(d);
            check_consistent(&h, 33);
            let n = 1u64 << d;
            for directed in [false, true] {
                // Independent of the code under test: the number of differing bits of all pairs.
                let expected: Vec<(Node, Node)> = (0..n)
                    .flat_map(|u| (0..n).map(move |v| (u, v)))
                    .filter(|&(u, v)| (u ^ v).count_ones() == 1 && (directed || u < v))
                    .collect();
                let got: Vec<_> = h
                    .edges(&Context::new(directed))
                    .map(|e| (e.u, e.v))
                    .collect();
                assert_eq!(got, expected, "d={d} directed={directed}");
            }
        }
    }

    #[test]
    fn the_dimension_is_limited() {
        // Independent of the code under test: the order `2^dimension` in `u128`.
        let order = |d: u64| 1u128 << d;
        let max = u128::from(MAX_ORDER);
        assert!(order(MAX_DIMENSION) <= max && order(MAX_DIMENSION + 1) > max);
        assert_eq!(Hypercube::new(MAX_DIMENSION).validate(), Ok(()));
        for d in [MAX_DIMENSION + 1, u64::MAX] {
            let errors = Hypercube::new(d).validate().unwrap_err();
            assert!(matches!(
                errors.as_slice(),
                [Error::InvalidParam {
                    name: "dimension",
                    ..
                }]
            ));
        }
    }
}
