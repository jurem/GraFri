//! A spec determines the graph: the same spec gives the same graph, and a change of any part
//! gives another one.
//!
//! Run with `cargo run --example reproducible_spec`.

use grafri::{Complete, Expr, Gnp, Graph, Spec};

fn main() -> grafri::Result<()> {
    // Everything that matters is in the spec: the expression (with the stream of each random
    // leaf), the directedness and the master seed.
    let expr = Expr::new(Gnp::new(12, 0.25, 1)) | Complete::new(3);
    let spec = Spec::new().with_seed(2024).with_expr(expr);
    println!("{spec:?}");

    // Built twice, or on another machine, it is the same graph.
    let first = spec.build()?;
    let second = spec.clone().build()?;
    assert!(first.edges().eq(second.edges()));
    println!("seed 2024: {}", first.display().with_edges());

    // A different seed samples the same expression again.
    let other = spec.clone().with_seed(2025).build()?;
    println!("seed 2025: {}", other.display().with_edges());
    assert!(!first.edges().eq(other.edges()));

    // The same seed, the other directedness: an undirected graph is a subgraph of the directed one.
    let directed = spec.directed().build()?;
    println!("directed: {}", directed.display().with_edges());
    assert!(first.edges().all(|e| directed.has_edge(e.u, e.v)));
    Ok(())
}
