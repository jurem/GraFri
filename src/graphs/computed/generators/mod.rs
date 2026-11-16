//! Generators: terms made from parameters alone. Each is lazy and deterministic and has no
//! directedness of its own. `size` and `has_edge` are O(1) unless stated, and `edges` lists the
//! edges in canonical order. Constructors do not check their parameters:
//! [`validate`](crate::Term::validate) does. Random generators are pure functions of their
//! parameters and the seed.

mod circulant;
mod complete;
mod complete_bipartite;
mod complete_tree;
mod cycle;
mod edges;
mod gnp;
mod grid;
mod hypercube;
mod path;
mod random_recursive_tree;
mod star;
mod torus;
mod wheel;

pub use circulant::Circulant;
pub use complete::Complete;
pub use complete_bipartite::CompleteBipartite;
pub use complete_tree::CompleteTree;
pub use cycle::Cycle;
pub use edges::Edges;
pub use gnp::Gnp;
pub use grid::Grid;
pub use hypercube::Hypercube;
pub use path::Path;
pub use random_recursive_tree::RandomRecursiveTree;
pub use star::Star;
pub use torus::Torus;
pub use wheel::Wheel;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::check_order_limit;

    /// Every generator rejects an order above `MAX_ORDER`. A new generator is added here.
    #[test]
    fn every_generator_limits_its_order() {
        check_order_limit(Path::new);
        check_order_limit(Cycle::new);
        check_order_limit(Wheel::new);
        check_order_limit(Complete::new);
        check_order_limit(|order| Circulant::new(order, [1]));
        check_order_limit(|order| CompleteTree::new(2, order));
        check_order_limit(|order| Edges::new(order, []));
        check_order_limit(|order| Gnp::new(order, 0.5, 1));
        check_order_limit(|order| RandomRecursiveTree::new(order, 1));
        check_order_limit(|order| Star::new(order - 1));
        check_order_limit(|order| CompleteBipartite::new(order / 2, order - order / 2));
        check_order_limit(|order| Grid::new(1, order));
        check_order_limit(|order| Torus::new(order, 1));
    }
}
