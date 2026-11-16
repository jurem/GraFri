//! Computed labelings: a label is worked out when asked for, and nothing is stored per element.

mod boolean;
mod choice;
mod constant;
mod float;
mod int;

pub use boolean::BoolLabeling;
pub use choice::ChoiceLabeling;
pub use constant::ConstantLabeling;
pub use float::FloatLabeling;
pub use int::IntLabeling;
