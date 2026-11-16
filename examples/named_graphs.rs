//! The structured graphs: wheel, circulant, grid and torus.
//!
//! Run with `cargo run --example named_graphs`.

use grafri::{Circulant, Cycle, Expr, Graph, GraphOf, Grid, Spec, Torus, Wheel};

/// The undirected graph of `expr`.
fn build(expr: impl Into<Expr>) -> grafri::Result<GraphOf> {
    Ok(Spec::new().with_expr(expr).build()?)
}

fn main() -> grafri::Result<()> {
    // Each generator is an expression without a directedness: `Spec` fixes it.
    let wheel = build(Wheel::new(5))?;
    println!("wheel(5): {}", wheel.display().with_edges());

    // Directed, the spokes point away from the hub and the rim goes around.
    let directed = Spec::new().directed().with_expr(Wheel::new(5)).build()?;
    println!("wheel(5), directed: {}", directed.display().with_edges());

    // Node i is joined to i + j for each jump j (mod n). Jump 1 alone is the cycle.
    let circulant = build(Circulant::new(8, [1, 3]))?;
    println!("circulant(8, [1, 3]): {}", circulant.display().with_edges());
    let one_jump = build(Circulant::new(6, [1]))?;
    assert!(one_jump.edges().eq(build(Cycle::new(6))?.edges()));
    println!("circulant(6, [1]) is the cycle(6): ok");

    // Nodes are numbered row by row. A torus is a grid that wraps around.
    let grid = build(Grid::new(2, 3))?;
    println!("grid(2, 3): {}", grid.display().with_edges());
    let torus = build(Torus::new(3, 3))?;
    println!("torus(3, 3): {}", torus.display().with_edges());
    assert_eq!(torus.size(), 18);
    println!("torus(3, 3) has 18 edges: ok");
    Ok(())
}
