//! Computed graphs. A [`Term`] is a tree of [`generators`] (from parameters) and [`combinators`]
//! (from other terms), with no directedness. A [`GraphOf`] fixes one under a
//! [`Context`](crate::Context), and an [`Expr`] combines terms with operators.

pub mod combinators;
mod expr;
pub mod generators;
mod graph_of;
mod limits;
mod term;

pub use expr::Expr;
pub use graph_of::GraphOf;
pub use limits::MAX_ORDER;
pub(crate) use limits::check_order;
pub use term::Term;
