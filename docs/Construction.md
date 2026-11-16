# Construction workflow

How a graph is made from a spec, and what is checked where. Three rules hold: a constructor never fails, a check reports every problem at once, and nothing is read before it is checked. This is the target; the code is behind it (the checked type, plan, locations, warnings and a public `Spec::check` are not implemented yet).

| # | Pass | Context | Check | Fails with | Produces |
|---|---|---|---|---|---|
| 0 | **Construct** | no | nothing | never | an unchecked term |
| 1 | **Compose** | no | nothing | never | an `Expr` |
| 2 | **Validate** | no | parameters in range, and the order of a generator at most `MAX_ORDER` | `Errors`, all of them | nothing |
| 3 | **Specify** | creates it | nothing | never | a `Spec`: directedness, seed, expression, labelings |
| 4 | **Resolve** | yes | what reads the context: the order of a combinator at most `MAX_ORDER` | `Errors`, all of them | a `Checked` term |
| 5 | **Validate the labelings** | no | names, ranges, probabilities, variants, labels that fit their kind, keys, and each repeated name | `Error::Labels`, all of them | nothing |
| 6 | **Resolve the labelings** | the graph | stored labelings only: one label per node for a `DenseLabeling`, only elements of the graph for a `KeyedLabeling` | `Error::Labels`, all of them | nothing |
| 7 | **Plan** (optional) | yes | nothing | never | a simplified, precomputed term |
| 8 | **Bind** | yes | nothing; only a `Checked` term can be bound, and the graph is lazy | never | a `GraphOf` |
| 9 | **Read** | yes | nothing | never | edges and labels |

Each pass assumes the previous one passed. A failing pass reports all its own problems and stops the pipeline: resolve runs only if validate found nothing, and the labelings are checked (5 and 6) only if the term passed (2 and 4), so the two kinds of error are never merged. `Term::check(ctx)` is validate (2) then resolve (4). `Spec::build` runs both, and `Spec::build_labeled` then checks the labelings and binds (8). A public `Spec::check()` would run the checks (2, 4, 5 and 6) without building the graph.

## The passes

**Construct** is the `new` of the generators and combinators. It never fails and never panics. It may normalize what has one canonical form (the jumps of a `Circulant`: taken modulo the order, multiples of the order dropped, sorted without repeats; the pairs of `Edges`: sorted without repeats). It keeps every invalid parameter as it is, so that the check can report it.

**Compose** wraps a term in an `Expr`, which combines terms with the operators `|`, `&`, `-`, `+`, `*` and `!`.

**Specify** is `Spec::new().directed().with_seed(..).with_expr(..).with_*_labeling(..)`. The `Context` is made from the directedness and the seed when the spec is built.

**Validate** (`Term::validate`) checks what needs no context: parameters in range, probabilities, arity, edge endpoints, and that the order of a generator is at most `MAX_ORDER`.

**Resolve** (`Term::resolve`) checks what has to read the orders of the operands: that the order of a `DisjointUnion` or a `CartesianProduct` is at most `MAX_ORDER`. It reads an operand's order only after the operand's own `resolve` passed. Later it may also yield the exact size and a `has_loops` flag, which would spare `CartesianProduct::size` and `Complement::size` their pass over the operands' edges (an optimization, not a check).

**The order bound** keeps the arithmetic plain. `MAX_ORDER` is `u32::MAX`, and a graph on `n` nodes has at most `n^2` edges or arcs, loops included, which is below `2^64` for every valid order. So once the orders are in range, no node number, count or size can overflow, directed or undirected, and the size of a combinator is a plain sum or product.

**Validate the labelings** (`Labeling::validate`) checks each labeling without the context: the name, the range of an `Int`, the probability of a `Bool`, the variants of an enumeration, a label of a stored labeling that does not fit its kind and variants, a key used twice in a `KeyedLabeling`, and each name used twice on one side.

**Resolve the labelings** (`Labeling::resolve(&dyn Graph)`) checks what depends on the graph, for the stored labelings only. It takes the graph, which is already fixed under its context, and no `Context`: a `DenseLabeling` has one label per node, and a `KeyedLabeling` names only nodes or edges of the graph. A computed labeling has nothing to resolve. It takes the graph (`Graph` lives in `base/`), not a `Term`, so that `labels/` stays independent of `graphs/`.

**Plan** produces the term that is read: precomputed leaves (the folded jumps of a `Circulant`, the sorted edges of a `RandomRecursiveTree`, the undirected edges of `Edges`) and `simplify()` (the reverse of an undirected graph is its operand, a double complement, a union with itself). It replaces the per-call work and the cache for these cases.

**Bind** (`GraphOf::new`) and **read** (`edges`, `has_edge`, `size`, `node_label`) cannot fail, since an unchecked term is unrepresentable: the arithmetic of a read is plain, with no saturation and no wrapping, and a read never panics.

**Errors** carry their path in the tree (`Difference.left.Union.right.Gnp`), which the spec language turns into a source position. **Warnings** that do not fail the build come next to the errors (a jump ignored because it is a multiple of the order, `p = 0`, a graph so large that `edges` collects billions of edges).

## Which pass checks what

Every order is at most `MAX_ORDER`. A generator rejects a larger order in `validate`, whatever the directedness; a combinator whose order is a sum or a product rejects it in `resolve`, once it can read the orders.

| Type | `validate` | `resolve` |
|---|---|---|
| `Gnp` | `p` in `[0, 1]`, order | |
| `Edges` | each endpoint `< order`, order | |
| `CompleteTree` | `arity >= 1`, order | |
| `Path`, `Cycle`, `Wheel`, `Complete`, `Circulant`, `RandomRecursiveTree` | order | |
| `Star` | order, `leaves + 1` | |
| `Hypercube` | order, `2^dimension`, so dimension `<= 31` | |
| `CompleteBipartite` | order, `left + right` | |
| `Grid`, `Torus` | order, `rows * cols` | |
| `Union`, `Intersection`, `Difference`, `Complement`, `Reverse` | the operands' `validate` | the operands' `resolve` |
| `DisjointUnion` | the operands' `validate` | the operands' `resolve`, then the orders add up to at most `MAX_ORDER` |
| `CartesianProduct` | the operands' `validate` | the operands' `resolve`, then the orders multiply to at most `MAX_ORDER` |

**In front of the library,** once the spec language exists: text, tokens, statements, syntax tree, names resolved, kinds checked, lowered to a `Term` (streams assigned by appearance). Syntax and arity errors belong to that front end; value ranges to validate.
