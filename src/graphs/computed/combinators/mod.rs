//! Combinators: terms made from other terms, so they nest. Operands are shared
//! (`Arc<dyn Term>`, from any term with [`Term::shared`]).

mod cartesian_product;
mod complement;
mod difference;
mod disjoint_union;
mod intersection;
mod reverse;
mod union;

use crate::Context;
use crate::graphs::computed::Term;

/// The number of self-loops among the edges of `e`, found by reading them all.
fn loops(e: &dyn Term, ctx: &Context) -> u64 {
    e.edges(ctx).filter(|edge| edge.u == edge.v).count() as u64
}

pub use cartesian_product::CartesianProduct;
pub use complement::Complement;
pub use difference::Difference;
pub use disjoint_union::DisjointUnion;
pub use intersection::Intersection;
pub use reverse::Reverse;
pub use union::Union;
