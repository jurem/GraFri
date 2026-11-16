//! [`Complement`]: the complement of a term.

use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::Term;
use crate::graphs::computed::combinators::loops;
use crate::graphs::computed::generators::Complete;
use crate::graphs::{Edge, Errors, Node, possible_edges};

/// The complement of a graph: the same nodes, with exactly the non-loop node pairs that the operand
/// does not have. Self-loops of the operand are ignored, and the complement has none.
#[derive(Debug, Clone)]
pub struct Complement(Arc<dyn Term>);

impl Complement {
    /// The complement of `a`.
    pub fn new(a: impl Term + 'static) -> Complement {
        Complement(a.shared())
    }
}

impl Term for Complement {
    fn order(&self, ctx: &Context) -> u64 {
        self.0.order(ctx)
    }

    /// The pairs of nodes minus the operand's edges that are not loops. Reads the operand's edges
    /// once to find its loops, so it is O(operand size).
    fn size(&self, ctx: &Context) -> u64 {
        let possible = Complete::new(self.order(ctx)).size(ctx);
        possible - (self.0.size(ctx) - loops(&*self.0, ctx))
    }

    /// Merges the operand's sorted edges with the sorted list of all pairs, in O(order² + operand
    /// size).
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        let possible = possible_edges(self.order(ctx), ctx.directed);
        let mut a = self.0.edges(ctx).peekable();
        Box::new(possible.filter(move |edge| {
            while a.next_if(|e| e < edge).is_some() {}
            a.next_if_eq(edge).is_none()
        }))
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        u != v && self.has_node(ctx, u) && self.has_node(ctx, v) && !self.0.has_edge(ctx, u, v)
    }

    fn validate(&self) -> Result<(), Errors> {
        self.0.validate()
    }

    fn resolve(&self, ctx: &Context) -> Result<(), Errors> {
        self.0.resolve(ctx)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::testing::{all_graphs, check_consistent, edges, pairs_of};

    #[test]
    fn matches_the_definition() {
        for directed in [false, true] {
            for graph in all_graphs(3, directed) {
                let present: BTreeSet<_> = graph.iter().copied().collect();
                // Independent of the code under test: the non-loop pairs the operand lacks.
                let expected: Vec<(Node, Node)> = (0..3)
                    .flat_map(|u| (0..3).map(move |v| (u, v)))
                    .filter(|&(u, v)| u != v && (directed || u < v) && !present.contains(&(u, v)))
                    .collect();
                let c = Complement::new(edges(3, &graph));
                assert_eq!(
                    pairs_of(&c, directed),
                    expected,
                    "{graph:?} directed={directed}"
                );
                check_consistent(&c, 4);
            }
        }
    }
}
