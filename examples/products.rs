//! The Cartesian product: structured graphs made from smaller ones.
//!
//! Run with `cargo run --example products`.

use grafri::{Complete, Cycle, Edge, Expr, Graph, GraphOf, Grid, Hypercube, Path, Spec, Torus};

/// The undirected graph of `expr`.
fn build(expr: impl Into<Expr>) -> grafri::Result<GraphOf> {
    Ok(Spec::new().with_expr(expr).build()?)
}

/// The edges of the undirected graph of `expr`.
fn edges_of(expr: impl Into<Expr>) -> grafri::Result<Vec<Edge>> {
    Ok(build(expr)?.edges().collect())
}

fn main() -> grafri::Result<()> {
    // `*` is the Cartesian product, as in the spec language. The node (u, v) of the product is
    // numbered `u * order(b) + v`.
    let grid = build(Expr::new(Path::new(2)) * Path::new(3))?;
    println!("path(2) * path(3): {}", grid.display().with_edges());

    // The grid is the product of two paths, and the torus the product of two cycles.
    assert_eq!(
        edges_of(Expr::new(Path::new(4)) * Path::new(5))?,
        edges_of(Grid::new(4, 5))?
    );
    assert_eq!(
        edges_of(Expr::new(Cycle::new(4)) * Cycle::new(5))?,
        edges_of(Torus::new(4, 5))?
    );
    println!("path * path is the grid, cycle * cycle is the torus: ok");

    // Multiplying by K2 adds a dimension to a hypercube.
    let cube = build(Expr::new(Hypercube::new(2)) * Complete::new(2))?;
    println!("hypercube(2) * k2: {}", cube.display().with_edges());
    assert!(cube.edges().eq(build(Hypercube::new(3))?.edges()));
    println!("hypercube(2) * k2 is the hypercube(3): ok");

    // Directed, the arcs follow the arcs of the operands.
    let directed = Spec::new()
        .directed()
        .with_expr(Expr::new(Path::new(2)) * Path::new(2))
        .build()?;
    println!("directed path * path: {}", directed.display().with_edges());
    Ok(())
}
