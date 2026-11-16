# Open decisions and where things stand

A hand-over note: what is open, what is decided, and what to do next. Nothing under "Open" may be implemented without an explicit go-ahead (see `CLAUDE.md`).

## Open

### The spec language

The design is in `docs/Spec-language.md` (flat one-line commands, Rust's operator precedence, labels at the root). Still open there:

- **The binding statement:** `name = expr` (the working form in the doc), `name let expr`, or `name is expr`. The user wanted to "sleep on it". The global default seed follows it (`seed = 42`, or a distinct command such as `default_seed 42`).
- The clash of `edges` (command) with `edges(...)` (generator), and of `edge` with `edges`, a letter apart: the user wants to rename something later.
- Whether `order` should be named for what it does (at least `n` nodes). Kept for now.
- **Syntax of `Reverse`:** it exists in the library but not in the language, and has no operator (`-` is the difference, `~` the complement). It needs a call form such as `reverse(a)`, the first call that takes an expression (so far, calls are generators with numeric arguments). It is not in the doc yet.
- Output commands, an `end` marker for blocks, `name.edges` as an expression.
- Label details: the default stream hash, `const`, `stream=` on a label generator, string generators, and negative numbers in arguments (a proposal is in the doc).
- Next implementation step: **the lexer** (statements end at a newline or `;`, newlines allowed inside `(...)` and `[...]` only).

### Labels

- A combined `LabeledGraph` trait (`Graph + NodeLabels + EdgeLabels`) for the language layer.

### Smaller items

- `Term::order` keeps its `ctx` on purpose, so that a generator whose number of nodes is random (a branching-process tree, site percolation, a Poisson number of points) can be added later; those are non-local and would use the cache. Such a generator can check its order in `Term::resolve`, which has the context.
- `Context` stays `Clone` and not `Copy` because it holds the cache. Only `RandomRecursiveTree` uses it now, to keep its edges sorted between `edges()` calls (saves work, never affects correctness). If that sort is done another way, the cache can go and `Copy` can be reconsidered.
- Later: `simplify()` and a `describe`-style hook on `Term` (printing a spec back). A default body makes adding either non-breaking.

## Decided (so that it is not reopened)

- `Node = u64`, an alias and not a newtype. `Edge { pub u: Node, pub v: Node }`. `has_edge(u, v)` everywhere. `Graph` and `Term` both have `has_node`; `is_node` / `is_edge` / `is_possible_edge` were left out.
- **Terms are direction-neutral.** A `Context { directed, seed, cache }` fixes the directedness and seed. A split of `directed` out of `Context` is discussed in `Ideas.md`.
- **`Expr` and `Spec`.** No session. `Expr` is only the operator wrapper around an `Arc<dyn Term>`: no leaf functions and no `build`; generators are wrapped by `Expr::new` or `From`, or used directly as the right operand. Operators exist for the language and for `Expr` only, not for raw terms (the orphan rules forbid it anyway). `Spec` is the only root and the counterpart of a graph block; it is an immutable value, so one spec gives both directions. A spec without an expression is `Error::MissingExpression` at `build` (an empty graph would hide a forgotten `with_expr`), and `Spec::new(expr)` was rejected because it hid the settings behind the expression. A session that owns the settings was dropped: its only job was numbering the random streams. Provided combinator methods on `Term` were sketched and not added.
- **Streams of random leaves are explicit** (`Gnp::new(n, p, stream)`). Numbering "by appearance" is the job of the language front end, which compiles it to explicit streams. A counter helper (`Streams`) was considered and not added.
- **Constructors are infallible; checking is separate.** `Term::validate` (no context), `Term::resolve(ctx)` and `Term::check(ctx)` (both, `resolve` only if `validate` found nothing) return `Result<(), Errors>`, where `Errors` is a non-empty list. `Spec::build` checks the whole tree once and returns every error. Reading an unchecked term is a misuse: the result is unspecified and may panic or wrap. `GraphOf::new` is the unchecked low-level way; `Term::apply` and `Expr::build(ctx)` were removed. What each pass checks is in `docs/Construction.md`.
- **Labels are read under a `Context`**, like terms. A computed labeling has a **stream** and no seed: its labels are `f(element.fold(stream_seed(ctx.seed, stream)))`, so the master seed re-samples labels together with the structure. The default stream is `random::name_stream(name)` (no element type in the hash, since nodes and edges are keyed differently anyway); `with_stream(k)` overrides it. Its reference vectors come from an independent Python implementation. A stored labeling ignores the context.
- **Labeling errors.** Constructors of labelings are infallible; `Labeling::validate()` reports every invalid parameter as a `labels::Errors` (for a stored labeling also the labels that do not fit its kind and variants, and the repeated keys of a `KeyedLabeling`), and `Labeling::resolve(&dyn Graph)` what does not fit the graph. `Spec::build_labeled` works in three phases: the errors of the expression (`Error::Check`); if there are none, those of `validate` of all the labelings together with every repeated name (`Error::Labels`); and then those of `resolve`. A phase runs only if the one before it found nothing, so the kinds are never merged.
- Labels: five kinds (`Bool`, `Int`, `Enum` (an index; the labeling keeps the variant names), `Float`, `String`; the text of a label comes from `Labeling::format`); node and edge labels are separate and exist only at the root, never inside an `Expr`, so that the operators never meet them. Stored: `DenseLabeling`, `KeyedLabeling<E>`; totality is not checked in `Labeled`.
- Combinators are non-generic tuple structs over `Arc<dyn Term>` (`Arc`, not `Box` or `Rc`); `Term: Debug + Send + Sync`, `Labeling<E>: Send + Sync`.
- `Gnp` decides each pair by a hash; the sequential `GnpSequential` was removed (source in `obsolete/gnp_sequential.rs`).
- Modules: `base/` (`Graph`, `Node`, `Edge`, `Context` and the generic `Errors<E>`, shared by `graphs/` and `labels/`; it depends only on `random/`), `graphs/` and `labels/` (each with `computed/` and `stored/`; `labels/` is independent of `graphs/`), `labeled/` and `spec.rs` (join them), `random/` and `error.rs` (the aggregate of the per-area errors).
- The language: operators `~` `*` `+ -` `&` `|` (Rust's precedence, tightest first), `+` is the disjoint union; flat commands of the shape *name, command, arguments*; a graph block is replaced by commands on a declared graph (`dg digraph`, `dg node color choice(...)`); labels at the root only.
