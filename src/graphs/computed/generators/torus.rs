//! [`Torus`]: the torus graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The torus on `rows * cols` nodes: the [`Grid`](super::Grid) with the first and last column
/// joined and the first and last row joined. The nodes are numbered row by row, as in the grid.
/// Directed, the arcs go right and down, wrapping around.
///
/// No self-loops, so a direction with one line adds no edges. A direction with two lines makes a
/// single edge if undirected, and two opposite arcs if directed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Torus {
    rows: u64,
    cols: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Torus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Torus")
            .field(&self.rows)
            .field(&self.cols)
            .finish()
    }
}

impl Torus {
    /// The torus with the given numbers of rows and columns.
    pub fn new(rows: u64, cols: u64) -> Torus {
        Torus { rows, cols }
    }

    /// The nodes to the right, to the left, below and above `u`, as `[right, left, below, above]`;
    /// some may be `u` itself or repeat.
    fn around(&self, u: Node) -> [Node; 4] {
        let (rows, cols) = (self.rows, self.cols);
        let (r, c) = (u / cols, u % cols);
        let at = |r: u64, c: u64| r * cols + c;
        let (next_c, prev_c) = (
            if c + 1 == cols { 0 } else { c + 1 },
            if c == 0 { cols - 1 } else { c - 1 },
        );
        let (next_r, prev_r) = (
            if r + 1 == rows { 0 } else { r + 1 },
            if r == 0 { rows - 1 } else { r - 1 },
        );
        [at(r, next_c), at(r, prev_c), at(next_r, c), at(prev_r, c)]
    }
}

/// The number of edges (or arcs) that close a line of `len` nodes: `len`, but 1 for `len = 2` if
/// undirected, and none for `len = 1`.
fn line(len: u64, directed: bool) -> u64 {
    match len {
        0 | 1 => 0,
        2 if !directed => 1,
        len => len,
    }
}

impl Term for Torus {
    fn order(&self, _ctx: &Context) -> u64 {
        self.rows * self.cols
    }

    fn size(&self, ctx: &Context) -> u64 {
        let (rows, cols) = (self.rows, self.cols);
        rows * line(cols, ctx.directed) + cols * line(rows, ctx.directed)
    }

    /// O(1) per node.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let directed = ctx.directed;
        Box::new((0..self.order(ctx)).flat_map(move |u| {
            // The edges that start at `u`, in order: the arcs to the right and below if
            // directed; the neighbors above `u` if undirected (each edge is listed at its
            // smaller end). The same neighbor can come up twice, and `u` itself too.
            let [right, left, below, above] = self.around(u);
            let mut ends = if directed {
                vec![right, below]
            } else {
                vec![right, left, below, above]
            };
            ends.retain(|&v| v != u && (directed || v > u));
            ends.sort_unstable();
            ends.dedup();
            ends.into_iter().map(move |v| Edge::directed(u, v))
        }))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        if u == v || !self.has_node(ctx, u) || !self.has_node(ctx, v) {
            return false;
        }
        let [right, left, below, above] = self.around(u);
        if ctx.directed {
            v == right || v == below
        } else {
            v == right || v == left || v == below || v == above
        }
    }

    /// The order, `rows * cols`, must be at most [`MAX_ORDER`](crate::MAX_ORDER).
    fn validate(&self) -> Result<(), Errors> {
        let order = u128::from(self.rows) * u128::from(self.cols);
        check_order("rows, cols", order)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::graphs::Graph;
    use crate::graphs::computed::generators::Cycle;
    use crate::testing::{build_graph, check_consistent, check_order_limit, graph_pairs};

    #[test]
    fn undirected_torus_3_by_3() {
        let g = build_graph(Torus::new(3, 3), false);
        assert_eq!(
            graph_pairs(&g),
            [
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 6),
                (1, 2),
                (1, 4),
                (1, 7),
                (2, 5),
                (2, 8),
                (3, 4),
                (3, 5),
                (3, 6),
                (4, 5),
                (4, 7),
                (5, 8),
                (6, 7),
                (6, 8),
                (7, 8)
            ]
        );
        assert_eq!((g.order(), g.size()), (9, 18));
    }

    #[test]
    fn directed_torus_2_by_3_wraps_right_and_down() {
        let g = build_graph(Torus::new(2, 3), true);
        assert_eq!(
            graph_pairs(&g),
            [
                (0, 1),
                (0, 3),
                (1, 2),
                (1, 4),
                (2, 0),
                (2, 5),
                (3, 0),
                (3, 4),
                (4, 1),
                (4, 5),
                (5, 2),
                (5, 3)
            ]
        );
        assert_eq!(g.size(), 12);
    }

    #[test]
    fn one_row_is_a_cycle() {
        for directed in [false, true] {
            let ctx = Context::new(directed);
            for n in 0..8 {
                let cycle: Vec<Edge> = Cycle::new(n).edges(&ctx).collect();
                assert_eq!(Torus::new(1, n).edges(&ctx).collect::<Vec<_>>(), cycle);
                assert_eq!(Torus::new(n, 1).edges(&ctx).collect::<Vec<_>>(), cycle);
            }
        }
    }

    #[test]
    fn matches_the_definition_for_small_sizes() {
        for rows in 0..6 {
            for cols in 0..6 {
                let n = rows * cols;
                check_consistent(&Torus::new(rows, cols), 10);
                for directed in [false, true] {
                    let ctx = Context::new(directed);
                    // Independent of the code under test: the row and column of each end of all
                    // pairs, wrapping.
                    let step = |u: u64, v: u64| {
                        let ((r1, c1), (r2, c2)) = ((u / cols, u % cols), (v / cols, v % cols));
                        (r1 == r2 && (c1 + 1) % cols == c2) || (c1 == c2 && (r1 + 1) % rows == r2)
                    };
                    let expected: Vec<Edge> = (0..n)
                        .flat_map(|u| (0..n).map(move |v| (u, v)))
                        .filter(|&(u, v)| u != v && (step(u, v) || (!directed && step(v, u))))
                        .map(|(u, v)| Edge::new(directed, u, v))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    assert_eq!(
                        Torus::new(rows, cols).edges(&ctx).collect::<Vec<_>>(),
                        expected,
                        "{rows}x{cols} directed={directed}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_order_is_limited() {
        check_order_limit(|order| Torus::new(1, order));
        check_order_limit(|order| Torus::new(order, 1));
        assert_eq!(Torus::new(65535, 65537).validate(), Ok(()));
        assert!(Torus::new(65536, 65536).validate().is_err());
        assert!(Torus::new(u64::MAX, u64::MAX).validate().is_err());
    }
}
