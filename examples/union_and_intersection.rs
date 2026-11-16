//! Union and intersection of a random graph and its complement.
//!
//! Run with `cargo run --example union_and_intersection`.

use grafri::{Complete, Expr, Gnp, Graph, GraphOf, Spec};

/// The undirected graph of `expr` under the master seed 1.
fn build(expr: impl Into<Expr>) -> grafri::Result<GraphOf> {
    Ok(Spec::new().with_seed(1).with_expr(expr).build()?)
}

fn main() -> grafri::Result<()> {
    // `&g` works as an operand, so the same random graph can be reused; it is the same graph each
    // time.
    let g = Expr::new(Gnp::new(6, 0.4, 1));
    println!("g: {}", build(&g)?.display().with_edges());
    println!("complement: {}", build(!&g)?.display().with_edges());

    // A graph and its complement together make the complete graph, and share no edge.
    let union = build(&g | !&g)?;
    let intersection = build(&g & !&g)?;
    println!("union: {}", union.display().with_edges());
    println!("intersection: {}", intersection.display().with_edges());

    assert!(union.edges().eq(build(Complete::new(6))?.edges()));
    assert_eq!(intersection.size(), 0);
    Ok(())
}
