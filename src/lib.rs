//! grafri: reproducible graph generation.
//!
//! A graph is either [`computed`](graphs::computed) from a [`Term`], which has no directedness, or
//! [`stored`](graphs::stored). An [`Expr`] combines terms, and a [`Spec`] fixes the
//! directedness, the seed and the labels, checks the expression and builds the graph.
//!
//! [`labels`] is independent of graphs: named, typed data for any elements. [`labeled`] joins the
//! two.

mod base;
pub mod error;
pub mod graphs;
pub mod labeled;
pub mod labels;
pub mod random;
pub mod spec;
#[cfg(test)]
mod testing;

pub use base::errors;
pub use base::{Context, Edge, Graph, GraphDisplay, Node};
pub use error::{Error, Result};
pub use graphs::Errors;
pub use graphs::computed::combinators::{
    CartesianProduct, Complement, Difference, DisjointUnion, Intersection, Reverse, Union,
};
pub use graphs::computed::generators::{
    Circulant, Complete, CompleteBipartite, CompleteTree, Cycle, Edges, Gnp, Grid, Hypercube, Path,
    RandomRecursiveTree, Star, Torus, Wheel,
};
pub use graphs::computed::{Expr, GraphOf, MAX_ORDER, Term};
pub use graphs::stored::EdgeList;
pub use labeled::{EdgeLabeling, EdgeLabels, Labeled, NodeLabeling, NodeLabels};
pub use labels::{
    BoolLabeling, ChoiceLabeling, ConstantLabeling, DenseLabeling, FloatLabeling, IntLabeling,
    KeyedLabeling, Kind, Label, Labeling, Variants,
};
pub use random::{Key, Prng, SplitMix64, Xoshiro256StarStar};
pub use spec::Spec;
