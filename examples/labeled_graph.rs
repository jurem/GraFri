//! A graph with labels: colors on the nodes, weights on the edges.
//!
//! Run with `cargo run --example labeled_graph`.

use grafri::{ChoiceLabeling, EdgeLabels, Gnp, Graph, IntLabeling, NodeLabels, Spec};

fn main() -> grafri::Result<()> {
    let master_seed = 42;

    // Labelings attach only at the root, the spec. A computed labeling is a pure function of the
    // master seed, its stream (derived from the name) and the node or edge; nothing is stored.
    let g = Spec::new()
        .with_seed(master_seed)
        .with_expr(Gnp::new(8, 0.4, 1))
        .with_node_labeling(ChoiceLabeling::new("color", ["red", "green", "blue"]))
        .with_edge_labeling(IntLabeling::new("weight", 1..10))
        .build_labeled()?;

    // The definitions come in the order they were added.
    let node_labelings: Vec<_> = g.node_labelings().collect();
    let edge_labelings: Vec<_> = g.edge_labelings().collect();

    println!("{}", g.display());
    println!("nodes");
    for v in g.nodes() {
        for labeling in &node_labelings {
            let label = g
                .node_label(labeling.name(), v)
                .expect("a node of the graph");
            // An enumeration label prints as its variant name.
            let text = labeling.format(&label).expect("a valid label");
            println!("  {v}  {} = {text}", labeling.name());
        }
    }

    println!("edges");
    for e in g.edges() {
        for labeling in &edge_labelings {
            let label = g
                .edge_label(labeling.name(), e.u, e.v)
                .expect("an edge of the graph");
            let text = labeling.format(&label).expect("a valid label");
            println!("  {}-{}  {} = {text}", e.u, e.v, labeling.name());
        }
    }

    // A pair that is not an edge has no label.
    assert_eq!(g.edge_label("weight", 0, 0), None);
    Ok(())
}
