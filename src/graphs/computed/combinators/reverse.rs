//! [`Reverse`]: the arcs of a term turned around, itself a term.

use std::sync::Arc;

use crate::Context;
use crate::graphs::computed::Term;
use crate::graphs::{Edge, Errors, Node};

/// The reverse of a graph: every arc `u -> v` of the operand becomes `v -> u`. Undirected, the
/// result is the operand unchanged.
#[derive(Debug, Clone)]
pub struct Reverse(Arc<dyn Term>);

impl Reverse {
    /// The reverse of `a`.
    pub fn new(a: impl Term + 'static) -> Reverse {
        Reverse(a.shared())
    }
}

impl Term for Reverse {
    fn order(&self, ctx: &Context) -> u64 {
        self.0.order(ctx)
    }

    fn size(&self, ctx: &Context) -> u64 {
        self.0.size(ctx)
    }

    /// Directed, collects, turns and sorts the operand's edges on each call, in O(size log size)
    /// time and O(size) memory.
    fn edges(&self, ctx: &Context) -> Box<dyn Iterator<Item = Edge> + '_> {
        if !ctx.directed {
            return self.0.edges(ctx);
        }
        let mut turned: Vec<Edge> = self
            .0
            .edges(ctx)
            .map(|e| Edge::directed(e.v, e.u))
            .collect();
        turned.sort_unstable();
        Box::new(turned.into_iter())
    }

    fn has_edge(&self, ctx: &Context, u: Node, v: Node) -> bool {
        self.0.has_edge(ctx, v, u)
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
    use super::*;
    use crate::graphs::computed::combinators::{Difference, Intersection, Union};
    use crate::graphs::computed::generators::{Cycle, Path, Star};
    use crate::testing::{check_consistent, edges, pairs_of};

    #[test]
    fn directed_arcs_are_turned_and_sorted() {
        let g = edges(4, &[(0, 3), (1, 0), (2, 0), (2, 3)]);
        assert_eq!(
            pairs_of(&Reverse::new(g), true),
            [(0, 1), (0, 2), (3, 0), (3, 2)]
        );
    }

    #[test]
    fn undirected_is_the_operand_itself() {
        let g = edges(4, &[(0, 3), (1, 0), (2, 3)]);
        let expected = pairs_of(&g, false);
        assert_eq!(pairs_of(&Reverse::new(g), false), expected);
    }

    #[test]
    fn matches_the_definition_for_small_graphs() {
        for n in 0..6 {
            let operands: [Box<dyn Term>; 3] = [
                Box::new(Path::new(n)),
                Box::new(Cycle::new(n)),
                Box::new(Star::new(n)),
            ];
            for operand in operands {
                let n = operand.order(&Context::undirected());
                let r = Reverse::new(operand);
                check_consistent(&r, 7);
                for directed in [false, true] {
                    let ctx = Context::new(directed);
                    // Independent of the code under test: an arc of the reverse is the opposite arc
                    // of the operand.
                    let expected: Vec<(Node, Node)> = (0..n)
                        .flat_map(|u| (0..n).map(move |v| (u, v)))
                        .filter(|&(u, v)| r.0.has_edge(&ctx, v, u) && (directed || u < v))
                        .collect();
                    assert_eq!(
                        pairs_of(&r, directed),
                        expected,
                        "n={n} directed={directed}"
                    );
                }
            }
        }
    }

    #[test]
    fn reversing_twice_gives_the_operand_back() {
        let g = edges(5, &[(0, 3), (1, 0), (4, 2), (2, 2)]);
        let twice = Reverse::new(Reverse::new(g.clone()));
        for directed in [false, true] {
            assert_eq!(pairs_of(&twice, directed), pairs_of(&g, directed));
        }
    }

    #[test]
    fn union_with_the_reverse_is_symmetric_and_the_intersection_is_mutual() {
        let g: Arc<dyn Term> = Arc::new(edges(4, &[(0, 1), (1, 0), (2, 3), (1, 2)]));
        let both = Union::new(Arc::clone(&g), Reverse::new(Arc::clone(&g)));
        assert_eq!(
            pairs_of(&both, true),
            [(0, 1), (1, 0), (1, 2), (2, 1), (2, 3), (3, 2)]
        );
        let mutual = Intersection::new(Arc::clone(&g), Reverse::new(Arc::clone(&g)));
        assert_eq!(pairs_of(&mutual, true), [(0, 1), (1, 0)]);
        let one_way = Difference::new(Arc::clone(&g), Reverse::new(g));
        assert_eq!(pairs_of(&one_way, true), [(1, 2), (2, 3)]);
    }
}
