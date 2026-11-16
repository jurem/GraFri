//! Random graphs G(n,p), and combining them.
//!
//! Run with `cargo run --example random_graphs`.

use grafri::{Expr, Gnp, Graph, GraphOf, Spec};

/// The undirected graph of `expr` under the master `seed`.
fn at_seed(expr: impl Into<Expr>, seed: u64) -> grafri::Result<GraphOf> {
    Ok(Spec::new().with_seed(seed).with_expr(expr).build()?)
}

fn main() -> grafri::Result<()> {
    // A random graph is determined by its parameters, its stream and the master seed of the spec.
    // Leaves with different streams are independent.
    let a = Expr::new(Gnp::new(8, 0.3, 1));
    let b = Expr::new(Gnp::new(8, 0.3, 2));

    let graph_a = at_seed(&a, 42)?;
    println!("a: {}", graph_a.display().with_edges());
    println!("b: {}", at_seed(&b, 42)?.display().with_edges());
    println!("a | b: {}", at_seed(&a | &b, 42)?.display().with_edges());
    println!("a - b: {}", at_seed(&a - &b, 42)?.display().with_edges());

    // The same parameters and stream are the same graph.
    let again = Expr::new(Gnp::new(8, 0.3, 1));
    assert!(graph_a.edges().eq(at_seed(again, 42)?.edges()));
    println!("same parameters, same graph: ok");

    // Another master seed gives another sample of the whole expression.
    println!(
        "a | b, seed 43: {}",
        at_seed(&a | &b, 43)?.display().with_edges()
    );
    Ok(())
}
