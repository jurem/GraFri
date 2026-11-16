//! Constructors do not check their parameters: `build` does, once, and reports every problem.
//!
//! Run with `cargo run --example checking_a_spec`.

use grafri::{
    BoolLabeling, Circulant, Complete, Edges, Error, Expr, Gnp, Graph, IntLabeling, Path, Spec,
};

fn main() {
    // Nothing is checked while the expression is made, so a chain with invalid parts still builds.
    let expr = Expr::new(Gnp::new(10, 1.5, 1))
        | Circulant::new(10, [0, 3, 12])
        | Edges::new(4, [(0, 9)])
        | Complete::new(3);

    // `build` finds all of them, in the order they appear in the expression.
    match Spec::new().with_expr(expr).build() {
        Ok(g) => println!("built a graph: {}", g.display()),
        Err(errors) => {
            println!("the spec is invalid:");
            for error in errors.as_slice() {
                println!("  {error}");
            }
        }
    }

    // The labelings are checked in the same way, all at once. The expression is checked first.
    let spec = Spec::new()
        .with_expr(Path::new(5))
        .with_node_labeling(IntLabeling::new("2n", 3..3)) // a bad name and an empty range
        .with_node_labeling(BoolLabeling::new("ok", 2.0)) // a bad probability
        .with_node_labeling(BoolLabeling::new("ok", 0.5)); // a name used twice
    match spec.build_labeled() {
        Err(Error::Labels(errors)) => {
            println!("the labelings are invalid:");
            for error in errors.as_slice() {
                println!("  {error}");
            }
        }
        Err(other) => println!("another error: {other}"),
        Ok(_) => println!("built a labeled graph"),
    }

    // A spec that was never given an expression cannot be built either.
    let errors = Spec::new().directed().build().unwrap_err();
    println!("without an expression: {}", errors.as_slice()[0]);

    // A valid spec builds.
    let ok = Spec::new()
        .with_expr(Expr::new(Gnp::new(10, 0.5, 1)) | Complete::new(3))
        .build();
    assert!(ok.is_ok());
}
