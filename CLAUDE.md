# CLAUDE.md

## Project

**grafri** is a Rust project for generating graphs. Core idea: **a graph is fully determined by its spec**. Instead of shipping large graph files, people share a short spec and regenerate the identical graph anywhere.

The project grows **gradually**. Do not add generators, formats or features that were not asked for. Clarity and a consistent pattern matter more than micro-optimization.

## Working agreement (important)

**Never edit files without an explicit go-ahead.** This is the main rule.

For any design question or change request (structure, data model, API, syntax, naming):
1. **First give a short comment on the design**: opinion, options, a recommendation, and what it would change (outputs, reference vectors, docs).
2. **Then ask before implementing**, and wait for an explicit yes
3. Implement only what was approved. Small, mechanical follow-ups of an approved change are fine.

What is *not* a go-ahead: a question ("can we remove X?"), a remark or complaint ("this is unclear"), an idea ("I would use X", "maybe Y?"), or any reaction to a proposal. These ask for an opinion. Answering a question never means changing a file. If it is unclear whether a reply is a yes, ask once more and do not edit.

Direct, unambiguous instructions ("rename X to Y", "remove Z") can be carried out right away. Do not bundle changes that were not requested into an approved one.

Go in small steps.

## Commands

```bash
export CARGO_TARGET_DIR="$HOME/.cargo-target/grafri"   # keep target/ out of OneDrive
cargo build
cargo test
cargo fmt --all && cargo clippy --all-targets -- -D warnings   # --all-targets includes the examples
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo run --example <name>                                      # examples/ must keep running
```

## Reproducibility contract (important)

Same spec + same grafri version ⇒ **bit-identical graph on every platform**.

- All randomness comes from `src/random/`, implemented in this crate and tested against reference vectors. Never use the `rand` crate's distributions, `gen_range`, `StdRng` or `thread_rng`. There are both directed and undirected, both part of the contract:
  - sequential generators (`Prng`: SplitMix64, xoshiro256**), each draw changes a state;
  - counter-based primitives (`mix`, `unit`, `bounded`, `stream_seed`, `Key`): the bits are a pure function of a seed and a key, so they do not depend on the order of the draws.
- Random structure and random labels are decided with the counter-based primitives: `Gnp` decides each pair by a hash, and a computed labeling is a function of `(seed, stream, element)`. The sequential `GnpSequential` (one draw per pair, in order) was removed; its source is kept in `obsolete/gnp_sequential.rs` for reference.
- Reference vectors for hash-based functions must come from an **independent implementation** (for example a small script in another language), never from the code under test.
- No `HashMap`/`HashSet` iteration in anything that affects output (use `Vec`/`BTreeMap`).
- No platform-dependent float math: avoid `ln`, `exp`, `powf`, trig. `+ - * /` and `sqrt` are fine. If a transcendental function is needed, use a pinned pure-Rust implementation. Ask first.
- Draw `u64`, never anything whose width depends on `usize`.
- Every term has an order of at most `MAX_ORDER` (`u32::MAX`), checked by `validate` and `resolve`. A graph then has fewer than `2^64` edges, so counts and sizes use plain arithmetic.
- **Before 1.0 anything may change**, including the output of existing specs, the API and the language; there are no golden-file tests and no changelog. But a **major or breaking change** needs careful consideration and your explicit confirmation first: changing an algorithm, the order of draws, any `random/` function, the meaning of a spec, or a public API. Say what it changes (outputs, reference vectors, docs, examples) and wait for a yes.
- A failing reference vector or test is a bug until shown otherwise: find the cause. Never edit a reference vector just to make a test pass.

## Vocabulary

Use these words consistently, in code, comments and docs:
- **node** (not vertex), `Node = u64`; the nodes of a graph of order `n` are `0..n`. An **edge** is `Edge { u, v }`; in an undirected graph `u <= v`.
- **directedness** of a graph means directed or undirected.
- A **term** (`Term`) describes a graph *without* a directedness, and an **expression** (`Expr`) combines terms with operators; a `Context` (directedness, seed) fixes it, giving a **graph** (`GraphOf`, which is a `Graph`). A stored graph is an `EdgeList`.
- A **label** is what a node or an edge carries; a `Kind` says what it can be; a `Labeling` has a name and a `Kind` and gives a label to each element. Labels are kept apart from the structure and attach only at the root (a `Labeled` graph), never inside a term.

## Conventions

- English for code, comments, docs, commit messages.
- Ask before adding any dependency.
- Library: no `unwrap`/`expect`/`panic!` on user input; return an error.
- Every public item has a doc comment (items of a crate-private module are exempt).
- Comments are short and describe only the item they document: no examples, and no restating of the code or of a sibling item. Lines are at most 100 columns; American spelling.
- In an `impl`, constructors come first (`new` first, then the others), then the other methods.
- `cargo fmt`, `cargo clippy --all-targets -D warnings` and `cargo doc` clean; all tests pass.
- `Term` and `Labeling` require `Send + Sync`. Terms are shared as `Arc<dyn Term>`.
- `base/` holds what `graphs/` and `labels/` share (`Graph`, `Node`, `Edge`, `Context`, the generic `Errors<E>`); it depends only on `random/`. `labels/` must stay independent of `graphs/` (it depends only on `base/` and `random/`). The language and the CLI may become separate crates.
- The repo is in a OneDrive folder whose path contains spaces: quote paths in shell commands.

## Design notes

Decisions that are not made yet, and drafts, live in `docs/`. Do not implement them without a go-ahead:
- `docs/Open-decisions.md`: open decisions and where things stand; read it first when resuming.
- `docs/Spec-language.md`: the spec language (draft; not implemented).
- `docs/Construction.md`: the target construction workflow (passes, context, checks). Ahead of the code: the checked type, plan, locations, warnings and a public `Spec::check` are not implemented yet.
- `docs/Ideas.md`: ideas put aside (the directedness as an argument of `build`, combinators that change the directedness).
- `obsolete/` holds retired code kept for reference only, not part of the build: `gnp_sequential.rs` (the sequential `Gnp`) and `edgelist.rs` (the first output-format writer).
