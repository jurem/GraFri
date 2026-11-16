# Ideas

Design ideas that were discussed and put aside for later. Each entry says what the idea is, what it would buy, and what it would cost, so that it can be picked up again without redoing the discussion. Nothing here is decided.

## The directedness of the graph as an argument of `build`, not part of `Context`

**Note.** Since `Spec` exists (`src/spec.rs`), the common case already states the directedness and the seed where the graph is made (`Spec::new().directed().with_seed(7).with_expr(e).build()`), and one spec gives both directions. This idea now only concerns the low-level `Context` API.

**The idea.** Directed or undirected is a property of the *graph*, while the seed is the environment it is generated in. State the directedness where the graph is made (`g.build(Directed, &ctx)`) and keep `Context` for the seed and the cache only.

**What it would buy.** One context serves both directions, so `examples/direction_neutral.rs` would not need two contexts with the same seed, and it matches the language, where the graph block sets the directedness (`dg digraph`) and the seed separately.

**What it would cost.** A bare `bool` is as unclear as `Context::new(true)`, so it needs a small enum whose name must not clash with `labels::Kind` (`Directedness`, `Orientation`). The directedness would also have to reach every node of the tree: either a second parameter on every `Term` method, or one bundled argument, which is today's `Context` under another name. A large mechanical edit for a regrouping of the same two values.

**A cheap alternative.** Keep `Context` and add `with_directed(bool)`, which returns a copy with another directedness that shares the seed and the cache, like `with_seed`.

**Ruled out:** putting the directedness in the type (`Term<K>` with marker types). It is very large, and the language could not choose the directedness at run time.

**Where it stands.** Put aside. Revisit if the two-context awkwardness shows up in real code once the language lowering is written, or if the context gets more fields.

## Combinators that change the directedness: `Underlying` and `Bidirected`

**The idea.** A term is read under the directedness of its `Context`. Two combinators would read their operand under the opposite directedness (same seed, shared cache, which is keyed by directedness) and convert at the boundary:

| Combinator | Read directed | Read undirected |
|---|---|---|
| `Underlying(e)` | `e` as it is | the edge `{u, v}` if `e` has the arc `u -> v` or `v -> u` |
| `Bidirected(e)` | both arcs for each edge of `e`'s undirected form | `e` as it is |

**What it would buy.** Mixing a directed and an undirected description in one spec. `Underlying` of a random graph is not the undirected random graph (it is denser: probability `1 - (1 - p)^2` for `Gnp`), and `Bidirected` is the only way to read an undirected graph as a symmetric digraph (`e | reverse(e)` does it only for generators whose directed form is oriented).

**What it would cost.** Two combinators of about 100 lines each (`edges` collects and sorts, `has_edge` asks the operand, `size` counts), and `Expr` methods. No change to the neutral design.

**Not taken:** a runtime directedness flag in every term (`Circulant::new(directed, ..)`). It loses "one term, both graphs" and touches every constructor.

**Where it stands.** Put aside until a spec needs to mix directedness.

## From the old design

Notes from the first version of the project, which had a command-line tool, string specs and a registry of generators, formats and PRNGs. Nothing here is decided, and none of it is to be done unless asked.

- More generators: random regular, G(n, m), Barabási–Albert, and others.
- More output formats: CSV, DIMACS, METIS, GraphViz DOT, JSON, GraphML, and readers that parse a format back into a graph.
- A writer should declare what it can represent and refuse a graph it cannot, rather than drop data silently. `obsolete/edgelist.rs` is the first version of a writer.
- A faster O(n + m) G(n, p), which needs a pinned `ln`.

**Models and names.** `gnp` is strictly the Gilbert model (1959): each possible edge is present independently with probability `p`. The name "Erdős–Rényi" originally refers to G(n, m), a graph chosen uniformly among all graphs with `n` nodes and exactly `m` edges (`m = 0` is the empty graph and `m = n(n-1)/2` is the complete graph). `gnp` follows the notation of the literature and of libraries such as NetworkX; a G(n, m) generator would be called `gnm`.
