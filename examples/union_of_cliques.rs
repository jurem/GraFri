//! A disjoint union of cliques: K3, K4 and K2 side by side.
//!
//! Run with `cargo run --example union_of_cliques`.

use grafri::{Complete, Expr, Graph, Spec};

fn main() -> grafri::Result<()> {
    // `+` is the disjoint union, as in the spec language.
    let expr = Expr::new(Complete::new(3)) + Complete::new(4) + Complete::new(2);
    println!("expr: {expr:?}");

    // A spec chooses the directedness (and the seed), checks the expression and gives the graph.
    let g = Spec::new().undirected().with_expr(expr).build()?;
    println!("graph: {g:?}");
    println!("{}", g.display().with_edges());
    Ok(())
}
