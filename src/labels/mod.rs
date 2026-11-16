//! Labels: named, typed data attached to the nodes or the edges of a graph.
//!
//! A [`Label`] is what a node or an edge carries, and its [`Kind`] says what it can be. A
//! [`Labeling`] has a name and a kind, and gives a label to each element of some type, so this
//! module knows nothing about graphs; it depends only on `random`, `context` and `errors`.
//! `computed` has the labelings worked out on demand, and `stored` those kept in memory. Both
//! depend only on the rest of this module.

mod computed;
mod error;
mod kind;
mod label;
mod labeling;
mod stored;
mod variants;

pub use computed::{BoolLabeling, ChoiceLabeling, ConstantLabeling, FloatLabeling, IntLabeling};
pub use error::{Error, Errors, Result};
pub use kind::Kind;
pub use label::Label;
pub use labeling::Labeling;
pub use stored::{DenseLabeling, KeyedLabeling};
pub use variants::Variants;
