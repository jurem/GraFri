//! One spec, many samples: the master seed re-samples the structure and the labels together.
//!
//! Run with `cargo run --example seed_sweep`.

use grafri::{ChoiceLabeling, Gnp, Graph, IntLabeling, NodeLabels, Spec};

fn main() -> grafri::Result<()> {
    // Everything but the seed is fixed: the expression and the labelings.
    let spec = Spec::new()
        .with_expr(Gnp::new(6, 0.4, 1))
        .with_node_labeling(ChoiceLabeling::new("color", ["red", "green", "blue"]))
        .with_edge_labeling(IntLabeling::new("weight", 1..10));

    for seed in [1, 2, 3] {
        let g = spec.clone().with_seed(seed).build_labeled()?;
        let color = g.node_labelings().next().expect("a node labeling");
        let colors: Vec<String> = g
            .nodes()
            .map(|v| {
                let label = g.node_label("color", v).expect("a node of the graph");
                color.format(&label).expect("a valid label")
            })
            .collect();
        println!("seed {seed}: {}, colors {}", g.display(), colors.join(" "));
    }

    // The same seed is the same sample.
    let again = spec.clone().with_seed(2).build_labeled()?;
    let first = spec.with_seed(2).build_labeled()?;
    assert!(first.edges().eq(again.edges()));
    assert!(
        first
            .nodes()
            .all(|v| first.node_label("color", v) == again.node_label("color", v))
    );
    Ok(())
}
