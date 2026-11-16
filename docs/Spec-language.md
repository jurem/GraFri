# Spec language (draft)

**Status: design in progress, not implemented yet.** It follows the current library: direction-neutral terms (`Term`) that a `Context` fixes into a graph, independent random streams per generator, and labels kept apart from the structure. Parts that are still proposals or undecided are marked **(proposal)** or **(undecided)** and are collected under "Open questions".

A spec is a list of one-line statements. Values are *expressions*: leaves are generators with parameters, inner nodes combine graphs.

```
gnp(100, 0.1) & ~complete(100)
```

A spec fully determines its graph: the same spec and the same grafri version give the identical graph on every platform.

A graph can also carry **labels** on its nodes and edges, and its directedness (directed or not) and seed are set with commands on a named graph:

```
struct = gnp(100, 0.1) | edges(10, [0-1, 2-3, 4-5])
seed = 42

dg digraph
dg order 100
dg edges struct
dg seed 7
dg node color    choice("red", "green", "blue")
dg node capacity int(1, 100)
dg edge weight   float(0, 1)
```

## Design in one paragraph

There are two sublanguages. **Expressions** (calls, operators, lists) describe structure and produce values. **Statements** are flat one-line commands of the shape *name, command, arguments*; they bind expressions to names, declare graphs, and set the directedness, seed and labels of a declared graph. There are no blocks, no nesting of commands, and no indentation.

## Overview of evaluation

```
text → tokens → statements → AST → computed expression (Term) → Spec → build → graph [+ labelings]
```

In the library, `Spec` (`src/spec.rs`) is the counterpart of a graph block: an expression, the directedness, the master seed and the labelings. The language front end compiles the automatic stream numbers into explicit ones (`Gnp::new(n, p, stream)`).

The directedness (directed or undirected), the master seed and the labels are not part of any generator or operation. They are set by the commands on a graph (or their defaults) and form the `Context` that the graph's expression is read under. Nothing inside an expression can mention them.

## Lexical rules

- **Whitespace** separates tokens. **Comments** start with `#` and run to the end of the line. Indentation is ignored.
- **Identifiers**: a letter or `_`, then letters, digits or `_`.
- **Integers**: decimal digits, a non-negative `u64`. **Floats**: digits, `.`, digits (`0.5`, `3.0`). `.5`, `5.` and exponents are not accepted. In an argument position (right after `(`, `,` or `=`), a `-` in front of a number literal negates it: `int(-5, 5)`. Anywhere else `-` is the difference operator. **(proposal)**
- **Booleans**: `true`, `false`.
- **Strings**: in double quotes, with the escapes `\"`, `\\`, `\n` and `\t`. A string cannot contain a raw newline.
- **Operators and punctuation**: `~ & | + - *`, `( ) [ ]`, `,`, `=`, `;`.
- **Statements** end at a newline or at `;`; the two are equivalent. A newline **inside `(...)` or `[...]`** is plain whitespace, so a long argument list or edge list can span lines. Nowhere else can a statement continue onto the next line: a statement is one line, and a long expression is split into several bindings.

## Expressions

- **Call**: `name(arg, ..., option=value, ...)`, e.g. `gnp(100, 0.1)`. **Required** parameters are positional, in the fixed order in which the generator declares them, and come first. **Optional** parameters (those with a default) are always named, come after the positional ones, and can be given in any order. A required parameter cannot be passed by name, and an optional one cannot be passed by position. The order of required parameters is part of a generator's definition and does not change once released; a new parameter is added as an optional one.
- **Operation**: binary `left op right` and prefix `~expr`. Precedence, from tightest to loosest: `~`, then `*`, then `+` and `-` (same level), then `&`, then `|`. This is Rust's order (and also that of Python and C): `+` and `-` bind tighter than `&`, which binds tighter than `|`. Binary operators are left-associative. Parentheses override all of it. So `a | b & c` is `a | (b & c)`, and `a - b | c` is `(a - b) | c`. The operators can be overloaded in Rust but their precedence cannot be changed, so keeping Rust's order means that the same expression means the same in a spec and in Rust code.
- **Variable**: a name bound by a binding statement (below).

An integer literal is accepted where a float is expected (`gnp(10, 1)`), not the other way round.

### Generators

| Call | Meaning |
|---|---|
| `complete(n)` | The complete graph on `n` nodes, no self-loops. |
| `path(n)` | The path `0 - 1 - ... - (n-1)`; directed, the arcs `i -> i+1`. |
| `star(k)` | `k` leaves: the hub `0` joined to each of `1..k`, so `k + 1` nodes; directed, the arcs `0 -> i`. |
| `complete_bipartite(a, b)` | The sides `0..a` and `a..a+b`, every node joined to every node of the other side; directed, the arcs go from the first side to the second. |
| `complete_tree(k, n)` | The tree filled level by level: node `i > 0` has the parent `(i-1) / k` (`k >= 1`); directed, the arcs go from the parent. `complete_tree(2, n)` is the binary tree. |
| `hypercube(d)` | `2^d` nodes, joined if they differ in one bit (`d <= 58`); directed, both arcs. |
| `random_recursive_tree(n)` | The random recursive tree: node `i > 0` is attached to a parent chosen uniformly from `0..i`; directed, the arcs go from the parent. Optional: `stream` (see "Randomness"). |
| `cycle(n)` | The path with the edge `(n-1) - 0` added; directed, the arcs `i -> (i+1) mod n`. No self-loops: `n < 2` has no edges, and an undirected `cycle(2)` is the single edge `0-1`. |
| `wheel(n)` | `n` nodes: the hub `0` joined to each of `1..n-1`, which form a cycle (so `wheel(5)` has a rim of four); directed, the arcs `0 -> i` and `i -> i+1` on the rim, the last back to `1`. `wheel(4)` is the complete graph on 4 nodes. |
| `circulant(n, [j, ...])` | Node `i` joined to `(i+j) mod n` for each jump `j`, e.g. `circulant(8, [1, 3])`; directed, the arcs `i -> (i+j) mod n`. Any jump is allowed and is taken modulo `n`; a multiple of `n` (such as 0) is ignored. Undirected, `j` and `n-j` are the same jump. `circulant(n, [1])` is `cycle(n)`. |
| `grid(r, c)` | `r * c` nodes numbered row by row (node `i*c + j` is in row `i`, column `j`), each joined to the one on its right and the one below; directed, the arcs go right and down. |
| `torus(r, c)` | The `grid(r, c)` with the first and last column joined, and the first and last row joined; directed, the arcs go right and down, wrapping around. A direction with one line adds no edges, and with two lines adds a single edge (two arcs if directed). |
| `gnp(n, p)` | The random graph G(`n`, `p`), each pair decided on its own by a hash; `p` in `[0, 1]`. Optional: `stream` (see "Randomness"). |
| `edges(n, [u-v, ...])` | `n` nodes and the given edges, e.g. `edges(5, [0-1, 1-2, 3-4])`. |

In `edges(...)`, `u-v` is an unordered edge in an undirected graph and the arc `u -> v` in a directed one; repeated edges count once; a node `>= n` is an error. The `-` inside the list is part of the edge syntax, not the difference operator.

Generators have no `directed` or `seed` parameter: both come from the commands on the graph.

### Operations

Let `n1` and `n2` be the node counts of the operands.

| Operation | Nodes | Edges |
|---|---|---|
| `a & b` | `0..min(n1, n2)` | edges present in both `a` and `b` |
| `a \| b` | `0..max(n1, n2)` | edges present in `a` or in `b` |
| `a - b` | `0..n1` | edges of `a` that are not in `b` |
| `a + b` | `0..n1 + n2` | edges of `a`, then edges of `b` with each node `v` renamed to `n1 + v` (disjoint union) |
| `~a` | same as `a` | every possible edge that `a` does not have; no self-loops |
| `a * b` | `n1 * n2` | Cartesian product (see below) |

For `&`, `|` and `-` the nodes of both operands are identified by index. `+` does not identify nodes: it places the two graphs side by side with no edge between them. Operations treat the edges as a **set**. The result is a simple graph, except that self-loops of explicit `edges(...)` operands are kept (the complement has none). The edges of the result are in lexicographic order, by the first end and then the second; for undirected graphs each edge is first written as `(smaller, larger)`. So `a & b` and `b & a` give the same graph.

Operations work on the structure only. Labels never take part in them.

**Cartesian product** (`a * b`; in the library as `CartesianProduct`, `Expr::cartesian_product` and `*`): the nodes are the pairs `(u, v)`, numbered `u * n2 + v`. Pairs `(u, v)` and `(u', v')` are joined if `u = u'` and `v ~ v'` in `b`, or `v = v'` and `u ~ u'` in `a`; directed, the arcs follow the arcs of the operands. The grid is the product of two paths, and the torus of two cycles. A self-loop of `a` at `u` gives a loop at `(u, v)` for every `v` (and likewise for `b`); a loop made by both counts once.

## Statements

Every statement is one line (or part of a line separated by `;`) and has one of these shapes. The first word is a name, except for the global `seed`.

| Shape | Meaning |
|---|---|
| `name = expr` | **Binding**: binds the *expression* to `name`. **(undecided: see below)** |
| `seed = n` | The default master seed (integer) for the graphs. At most once. **(undecided)** |
| `name digraph`, `name ungraph`, `name graph` | **Declares** a graph called `name`. |
| `name command arguments` | A **command** on the declared graph `name` (next section). |
| *expression* | A bare expression (a first token that is a call, `~`, `(` or a variable followed by an operator). |

- **A binding** binds the expression itself, so a random graph bound once is the same graph everywhere it is used: with `a = gnp(10, 0.5)`, the expression `a & ~a` uses one graph. Writing `gnp(10, 0.5) & ~gnp(10, 0.5)` is two independent graphs.
- **Names** are bound only once, by bindings and graph declarations together. It is an error to use a name before it is bound, and an error for a binding that is never used.
- **Telling the shapes apart:** a first identifier followed by `=` is a binding; a first identifier followed by another identifier is a command (the second word is the command, which can be a command word only in this position); anything else is a bare expression. Because commands are recognized by position, their words (`order`, `edges`, `node`, ...) are not reserved and can still be names or generators elsewhere. Only the global `seed` is special as a first word.
- **The result** of a spec is its one graph: the declared graph, or a bare expression if no graph is declared. A spec with a declared graph and a bare expression, or with two graphs, is an error for now (there would be two results); explicit output commands will lift this.

No loops or conditionals.

## Graph commands

A graph is declared with `name digraph`, `name ungraph` or `name graph`, and configured by commands that start with its name. Commands can come in any order after the declaration.

| Command | Meaning |
|---|---|
| `name directed` / `name undirected` | The directedness of the graph. Default: undirected (`digraph` and `ungraph` set it; a contradiction is an error). |
| `name edges expr` | The structure: any expression. Required, exactly once. |
| `name order n` | At least `n` nodes: the structure is united with `n` isolated nodes, i.e. `edges(n, []) \| <structure>`. |
| `name seed n` | The master seed for this graph; overrides the global `seed`. Default 0. |
| `name node label generator` | A node label (below). |
| `name edge label generator` | An edge label (below). |

- `name digraph` is the same as `name graph` followed by `name directed`, and `name ungraph` the same as `name graph` followed by `name undirected`.
- The order of the `node` and `edge` lines gives the order of the labels (for example, of the columns of an output).
- A graph is not an expression and cannot be an operand of any operation. Later, `name.edges` may give its structure as an expression, without the labels. **(future)**
- The word `edges` in `name edges expr` is a command, found by position; inside the expression it is still the generator `edges(...)`: `dg edges edges(10, [0-1])` is valid.

## Labels

A **label** is what a node or an edge carries under a name. Labels are kept apart from the structure and never take part in operations. Node labels and edge labels are separate, with separate name spaces.

- A label has a **kind**: `bool`, `int` (`i64`), `float` (`f64`, finite), `enum` or `string` (UTF-8). An `enum` label is an index into the names of its variants and prints as the name.
- `name node label generator` and `name edge label generator` define a *labeling*: a name, a directedness and a generator that gives every node (or edge) its label. A name defined twice for the same element type is an error. Every label is **total**: there are no missing values.
- **Generators**:

| Call | Labels |
|---|---|
| `const(x)` | the same label for every element; the directedness is that of `x` **(proposal)** |
| `bool(p)` | `true` with probability `p` |
| `int(a, b)` | a random integer in `a..b` (`b` excluded) |
| `float(a, b)` | a random float in `[a, b)` |
| `choice(v1, v2, ...)` | an `enum` whose variants are the arguments (distinct, non-empty strings), each equally likely |

- A random label is a **pure function of its element**: the label of a node `v` or an edge `(u, v)` is computed from the master seed, the labeling's stream and the element, by hashing. For an undirected edge the pair is `(smaller, larger)`. So a label does not depend on the order in which anything is read, and it is found in O(1) without generating the rest.
- The default stream of a labeling is a hash of its name (`color`, `weight`), `name_stream` in the library: `mix` folded over the number of bytes and then each byte of the name, from state 0. So adding, removing or reordering labelings never changes the structure or the other labelings; nodes and edges with the same name are still independent, because their keys differ. An explicit `stream=` on the generator overrides it.

## Randomness

- Structure: `gnp(n, p)` decides each pair on its own. With `seed' = stream_seed(seed, stream)`, the pair `(u, v)` is an edge exactly when `unit((u, v).fold(seed')) < p`. The undirected graph is a subgraph of the directed one with the same seed and stream.
- Every random generator has a **stream** number. Leaves with different streams are independent; two leaves with the same parameters and the same stream are the same graph. `seed' = seed + stream * GAMMA`, where `GAMMA` is SplitMix64's increment.
- By default, the random generators of the structure are numbered `0, 1, 2, ...` in the order in which they appear in the text, left to right, counting each textual occurrence. A generator inside a binding is counted at the binding, not at its uses. An explicit `stream=` overrides the number for that generator and does not change the numbering of the others. An explicit stream equal to an automatic number gives the same graph. Labelings do not use these numbers.
- The master seed of a graph is `name seed n` if given, else the global `seed`, else 0. Changing it gives another sample of the whole graph.
- Adding a random generator in the middle of a spec shifts the numbers of the later ones and so changes their graphs; use explicit `stream=` where that matters.
- The order in which operands are evaluated or queried never affects the result.
- The counter-based primitives (`mix`, `unit`, `bounded`) and the PRNG xoshiro256** are part of the library's reproducibility contract; changing them is a breaking change.

## Errors

A spec that does not parse or evaluate is rejected as a whole, before any graph is produced, with a message that names the line and column where possible. The arguments of the generators are validated by `Term::check`, which reports all of them at once:

- a syntax error, or a statement that continues onto a second line outside brackets;
- an unknown generator, command or option, a missing or surplus argument, an argument of the wrong type, or a value out of range (for example `p` outside `[0, 1]`, a node `>= n`, an empty `int` range);
- a name used before it is bound, bound twice, or never used; a command on a name that is not a graph, or before the graph is declared;
- a graph without `edges`, with a repeated `edges`, `order` or `seed` command, or with a contradicting directedness;
- a label defined twice, or `choice` with repeated variants;
- more than one result (two graphs, or a graph and a bare expression).

## Examples

```
complete(5)                              # K5
~gnp(20, 0.3)                            # complement of a random graph
complete(3) + complete(4) + complete(2)  # three cliques side by side
a = gnp(50, 0.1)
a & ~a                                   # always the empty graph

seed = 42
d digraph
d edges gnp(50, 0.1)
d node group choice("a", "b")
d edge weight int(1, 10)

# on one line
d digraph; d edges gnp(5, 0.5); d node c bool(0.5)
```

## Open questions

- **The binding statement (undecided).** `name = expr`; or `name let expr`, which keeps the shape *name, command, arguments* but reads backwards; or `name is expr`. The global default seed follows the same choice (`seed = 42`, or a distinct command such as `default_seed 42`).
- The clash of `edges` (command, generator) and `edge` (the label command), a letter apart: to be renamed.
- Whether `order` should be named for what it does (at least `n` nodes).
- Output commands, and the explicit result they bring; `name.edges` as an expression.
- The exact canonical printed form of a spec (whitespace, parameter order, defaults, which parentheses are kept), which a CLI that reports the spec it ran would need.
- How the CLI reports a randomly chosen seed (`--random-seed`); the library itself never reads the clock.
- Further operations.
- Labels: the `const` generator, the syntax of an explicit `stream=` on a label generator, string-valued generators (formatted names, functions of the node number), and negative numbers in arguments (as proposed above).
- Whether explicit graphs (`edges(...)`) can carry labels given in the spec.
