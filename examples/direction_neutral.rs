//! One expression, both directed and undirected: an expression has no direction of its own, the
//! spec it is read under decides.
//!
//! Run with `cargo run --example direction_neutral`.

use grafri::{Complete, Expr, Gnp, Graph, GraphOf, Spec};

/// The graphs of `spec`: undirected, then directed.
fn both(spec: &Spec) -> grafri::Result<(GraphOf, GraphOf)> {
    Ok((
        spec.clone().undirected().build()?,
        spec.clone().directed().build()?,
    ))
}

fn main() -> grafri::Result<()> {
    // A complete graph: each pair once if undirected, both arcs if directed.
    let (graph, digraph) = both(&Spec::new().with_expr(Complete::new(4)))?;
    println!("K4: {}", graph.display().with_edges());
    println!("K4: {}", digraph.display().with_edges());

    // A random graph decides each pair on its own, so the undirected graph is a
    // subgraph of the directed one with the same seed and stream.
    let g = Expr::new(Gnp::new(6, 0.4, 1));
    let (graph, digraph) = both(&Spec::new().with_seed(7).with_expr(&g))?;
    println!("gnp: {}", graph.display().with_edges());
    println!("gnp: {}", digraph.display().with_edges());
    assert!(graph.edges().all(|e| digraph.has_edge(e.u, e.v)));

    // Combinators are direction-neutral too: the complement of the same expression.
    let (graph, digraph) = both(&Spec::new().with_seed(7).with_expr(!&g))?;
    println!("complement: {}", graph.display().with_edges());
    println!("complement: {}", digraph.display().with_edges());
    Ok(())
}
