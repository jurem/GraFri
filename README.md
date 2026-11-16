# grafri

Reproducible graph generation in Rust.

A graph is fully determined by its **spec**: the expression that describes its structure (with the stream of each random part), the directedness, the master seed and the labels. Share the short spec instead of a large file, and anyone can regenerate the identical graph, bit for bit, on every platform. All randomness comes from `src/random/`, which is implemented in this crate and tested against reference vectors; nothing depends on a platform or on a version of another crate.

**Status:** an early library. The command-line tool (`src/main.rs`) is a stub, and the spec *language* (`docs/Spec-language.md`) is a draft that is not implemented yet.

## Example

```rust
use grafri::{Expr, ChoiceLabeling, Gnp, Graph, Spec, Torus};

fn main() -> grafri::Result<()> {
    // Say what kind of graph it is, then what it contains, then build it.
    let g = Spec::new()
        .directed()
        .with_seed(42)
        .with_expr(Expr::new(Gnp::new(100, 0.05, 1)) | Torus::new(10, 10))
        .with_node_labeling(ChoiceLabeling::new("color", ["red", "green", "blue"]))
        .build_labeled()?;
    println!("{} nodes, {} edges", g.order(), g.size());
    Ok(())
}
```

Changing the seed re-samples the random parts and the labels together; the same spec gives the same graph. More programs are in `examples/` (`cargo run --example <name>`).

## What there is

- **Terms** describe a graph's structure without a directedness (`Term`). They are *generators* made from parameters (`Complete`, `CompleteBipartite`, `CompleteKaryTree`, `Circulant`, `Cycle`, `Edges`, `Gnp`, `Grid`, `Hypercube`, `Path`, `RandomRecursiveTree`, `Star`, `Torus`, `Wheel`) and *combinators* made from other terms (union, intersection, difference, disjoint union, Cartesian product, complement, reverse). An `Expr` wraps a term so that combinators can be written with operators: `|` `&` `-` `+` `*` `!`.
- A **`Spec`** fixes the directedness, the seed and the labels, and builds the graph. One expression gives both a directed and an undirected graph. Parameters are not checked while a spec is made; `build` reports every invalid one at once.
- **Labels** are kept apart from the structure: computed labelings (`BoolLabeling`, `IntLabeling`, `FloatLabeling`, `ChoiceLabeling`, `ConstantLabeling`) are pure functions of the master seed, a stream and the node or edge, so nothing is stored; stored ones (`DenseLabeling`, `KeyedLabeling`) keep explicit values.
- **Stored graphs** (`EdgeList`) keep any graph explicitly.

## Design notes

- `CLAUDE.md`: the project's rules, vocabulary and the reproducibility contract.
- `docs/Open-decisions.md`: what is decided and what is open. Read it first when resuming.
- `docs/Spec-language.md`: the draft of the spec language.
- `docs/Ideas.md`: ideas put aside.

## License

Licensed under either of [Apache License, Version 2.0](License-Apache) or [MIT license](License-MIT) at your option.
