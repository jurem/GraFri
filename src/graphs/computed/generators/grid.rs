//! [`Grid`]: the grid graph.

use std::fmt;

use crate::Context;
use crate::graphs::computed::{Term, check_order};
use crate::graphs::{Edge, Errors, Node};

/// The grid on `rows * cols` nodes, numbered row by row: the node in row `r` and column `c` is
/// `r * cols + c`. It is joined to the node on its right and to the node below it, where they
/// exist. Directed, the arcs go right and down. No self-loops; a grid with no rows or no columns is
/// empty.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    rows: u64,
    cols: u64,
}

/// Shown as its constructor call.
impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Grid")
            .field(&self.rows)
            .field(&self.cols)
            .finish()
    }
}

impl Grid {
    /// The grid with the given numbers of rows and columns.
    pub fn new(rows: u64, cols: u64) -> Grid {
        Grid { rows, cols }
    }
}

impl Term for Grid {
    fn order(&self, _ctx: &Context) -> u64 {
        self.rows * self.cols
    }

    fn size(&self, _ctx: &Context) -> u64 {
        let (rows, cols) = (self.rows, self.cols);
        // The edges to the right, then the edges below.
        rows * cols.saturating_sub(1) + cols * rows.saturating_sub(1)
    }

    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        // `u < v`, so the same edges are canonical in both cases. For each `u`, the edge to
        // the right (`u + 1`) sorts before the one below (`u + cols`).
        let (cols, order) = (self.cols, self.order(ctx));
        Box::new((0..order).flat_map(move |u| {
            let right = (u % cols + 1 < cols).then(|| Edge::directed(u, u + 1));
            let below = (u < order - cols).then(|| Edge::directed(u, u + cols));
            right.into_iter().chain(below)
        }))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        let (u, v) = if ctx.directed || u <= v {
            (u, v)
        } else {
            (v, u)
        };
        if !(u < v && self.has_node(ctx, v)) {
            return false;
        }
        let gap = v - u;
        (gap == 1 && u % self.cols + 1 < self.cols) || gap == self.cols
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
    use crate::testing::{build_graph, check_consistent, check_order_limit, graph_pairs};

    #[test]
    fn grid_2_by_3() {
        for directed in [false, true] {
            let g = build_graph(Grid::new(2, 3), directed);
            assert_eq!(
                graph_pairs(&g),
                [(0, 1), (0, 3), (1, 2), (1, 4), (2, 5), (3, 4), (4, 5)]
            );
            assert_eq!((g.order(), g.size()), (6, 7));
        }
    }

    #[test]
    fn matches_the_definition_for_small_sizes() {
        for rows in 0..5 {
            for cols in 0..5 {
                let n = rows * cols;
                check_consistent(&Grid::new(rows, cols), 10);
                for directed in [false, true] {
                    let ctx = Context::new(directed);
                    // Independent of the code under test: the row and column of each end of all
                    // pairs.
                    let step = |u: u64, v: u64| {
                        let ((r1, c1), (r2, c2)) = ((u / cols, u % cols), (v / cols, v % cols));
                        (r1 == r2 && c1 + 1 == c2) || (c1 == c2 && r1 + 1 == r2)
                    };
                    let expected: Vec<Edge> = (0..n)
                        .flat_map(|u| (0..n).map(move |v| (u, v)))
                        .filter(|&(u, v)| step(u, v) || (!directed && step(v, u)))
                        .map(|(u, v)| Edge::new(directed, u, v))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    assert_eq!(
                        Grid::new(rows, cols).edges(&ctx).collect::<Vec<_>>(),
                        expected,
                        "{rows}x{cols} directed={directed}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_order_is_limited() {
        check_order_limit(|order| Grid::new(1, order));
        check_order_limit(|order| Grid::new(order, 1));
        // 65535 * 65537 = 2^32 - 1, and 65536 * 65536 = 2^32.
        assert_eq!(Grid::new(65535, 65537).validate(), Ok(()));
        assert!(Grid::new(65536, 65536).validate().is_err());
        assert!(Grid::new(u64::MAX, u64::MAX).validate().is_err());
    }
}
