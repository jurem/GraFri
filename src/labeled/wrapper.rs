//! [`Labeled`]: a graph with node labelings and edge labelings.

use std::sync::Arc;

use super::{EdgeLabeling, EdgeLabels, NodeLabeling, NodeLabels};
use crate::labels::{Error, Errors, Label, Labeling};
use crate::{Context, Edge, Graph, Node};

/// A graph together with any number of node labelings and edge labelings. It is a [`Graph`] itself,
/// forwarding every structural question to the graph, and answers label questions by name (see
/// [`NodeLabels`] and [`EdgeLabels`]).
///
/// A label exists only for an element of the graph: asking for a node outside it, or an edge it
/// does not have, gives `None`. Node labels and edge labels have separate name spaces. A
/// `DenseLabeling` must have one label per node, which is checked when it is added; a
/// `KeyedLabeling` may cover only some elements.
///
/// Computed labelings are read under a [`Context`]: its master seed decides their labels. The
/// default is `Context::default()`; set it with [`with_context`](Labeled::with_context). Stored
/// labelings ignore it.
#[derive(Clone)]
pub struct Labeled<G> {
    graph: G,
    ctx: Context,
    node_labelings: Vec<Arc<dyn NodeLabeling>>,
    edge_labelings: Vec<Arc<dyn EdgeLabeling>>,
}

impl<G: Graph> Labeled<G> {
    /// `graph`, with no labels, read under the default context.
    pub fn new(graph: G) -> Labeled<G> {
        Labeled {
            graph,
            ctx: Context::default(),
            node_labelings: Vec::new(),
            edge_labelings: Vec::new(),
        }
    }

    /// The graph with the given labelings, which the caller has already checked against it.
    pub(crate) fn from_checked(
        graph: G,
        ctx: Context,
        node_labelings: Vec<Arc<dyn NodeLabeling>>,
        edge_labelings: Vec<Arc<dyn EdgeLabeling>>,
    ) -> Labeled<G> {
        Labeled {
            graph,
            ctx,
            node_labelings,
            edge_labelings,
        }
    }

    /// The same graph with one more node labeling.
    ///
    /// # Errors
    /// The errors of [`Labeling::validate`](crate::Labeling::validate), then, if there are none, of
    /// [`Labeling::resolve`](crate::Labeling::resolve), and [`Error::DuplicateName`] if a node
    /// labeling with the same name is already there.
    pub fn with_node_labeling(
        mut self,
        labeling: impl NodeLabeling + 'static,
    ) -> Result<Self, Errors> {
        add::<dyn NodeLabeling, Node>(&self.graph, &mut self.node_labelings, Arc::new(labeling))?;
        Ok(self)
    }

    /// The same graph with one more edge labeling.
    ///
    /// # Errors
    /// The errors of [`Labeling::validate`](crate::Labeling::validate), then, if there are none, of
    /// [`Labeling::resolve`](crate::Labeling::resolve), and [`Error::DuplicateName`] if an edge
    /// labeling with the same name is already there.
    pub fn with_edge_labeling(
        mut self,
        labeling: impl EdgeLabeling + 'static,
    ) -> Result<Self, Errors> {
        add::<dyn EdgeLabeling, Edge>(&self.graph, &mut self.edge_labelings, Arc::new(labeling))?;
        Ok(self)
    }

    /// The same graph and labelings, read under `ctx` (a `Context` or a `&Context`).
    pub fn with_context(self, ctx: impl Into<Context>) -> Labeled<G> {
        Labeled {
            ctx: ctx.into(),
            ..self
        }
    }

    /// The context that the labelings are read under.
    pub fn context(&self) -> &Context {
        &self.ctx
    }

    /// The graph that is labeled.
    pub fn graph(&self) -> &G {
        &self.graph
    }
}

/// Adds `labeling` to `list` if it is valid, fits `graph` and has a new name.
fn add<L: ?Sized + Labeling<E>, E>(
    graph: &dyn Graph,
    list: &mut Vec<Arc<L>>,
    labeling: Arc<L>,
) -> Result<(), Errors> {
    labeling.validate()?;
    labeling.resolve(graph)?;
    let name = labeling.name();
    if list.iter().any(|l| l.name() == name) {
        return Err(Error::DuplicateName(name.to_string()).into());
    }
    list.push(labeling);
    Ok(())
}

/// The labeling called `name` in `list`.
fn find<'a, L: ?Sized + Labeling<E>, E>(list: &'a [Arc<L>], name: &str) -> Option<&'a L> {
    list.iter().find(|l| l.name() == name).map(|l| &**l)
}

impl<G: Graph> NodeLabels for Labeled<G> {
    fn node_labelings(&self) -> Box<dyn Iterator<Item = &dyn NodeLabeling> + '_> {
        Box::new(self.node_labelings.iter().map(|l| &**l))
    }

    fn node_label(&self, name: &str, v: Node) -> Option<Label> {
        if !self.graph.has_node(v) {
            return None;
        }
        find(&self.node_labelings, name)?.label(&self.ctx, v)
    }
}

impl<G: Graph> EdgeLabels for Labeled<G> {
    fn edge_labelings(&self) -> Box<dyn Iterator<Item = &dyn EdgeLabeling> + '_> {
        Box::new(self.edge_labelings.iter().map(|l| &**l))
    }

    fn edge_label(&self, name: &str, u: Node, v: Node) -> Option<Label> {
        if !self.graph.has_edge(u, v) {
            return None;
        }
        let edge = Edge::new(self.graph.is_directed(), u, v);
        find(&self.edge_labelings, name)?.label(&self.ctx, edge)
    }
}

impl<G: Graph> Graph for Labeled<G> {
    fn order(&self) -> u64 {
        self.graph.order()
    }

    fn nodes(&self) -> std::ops::Range<Node> {
        self.graph.nodes()
    }

    fn has_node(&self, v: Node) -> bool {
        self.graph.has_node(v)
    }

    fn size(&self) -> u64 {
        self.graph.size()
    }

    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        self.graph.edges()
    }

    fn has_edge(&self, u: Node, v: Node) -> bool {
        self.graph.has_edge(u, v)
    }

    fn is_directed(&self) -> bool {
        self.graph.is_directed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::computed::generators::Gnp;
    use crate::graphs::stored::EdgeList;
    use crate::labels::{BoolLabeling, DenseLabeling, IntLabeling, KeyedLabeling, Kind, Variants};
    use crate::testing::build_graph;

    /// The path 0 - 1 - 2, undirected or directed.
    fn path(directed: bool) -> EdgeList {
        EdgeList::new(3, directed, [(0, 1), (1, 2)]).unwrap()
    }

    fn colors() -> DenseLabeling {
        let variants = Variants::new(["red", "green"]);
        let labels = [Label::Enum(0), Label::Enum(1), Label::Enum(0)];
        DenseLabeling::enumeration("color", variants, labels)
    }

    fn weights(directed: bool) -> KeyedLabeling<Edge> {
        let entries = [
            (Edge::new(directed, 0, 1), Label::Int(10)),
            (Edge::new(directed, 1, 2), Label::Int(20)),
        ];
        KeyedLabeling::new("weight", Kind::Int, entries)
    }

    #[test]
    fn a_labeling_that_does_not_fit_the_graph_is_rejected() {
        let two = DenseLabeling::new("c", Kind::Int, [Label::Int(1), Label::Int(2)]);
        let errors = Labeled::new(path(false))
            .with_node_labeling(two)
            .err()
            .unwrap();
        assert_eq!(
            errors.as_slice(),
            [Error::LabelCount {
                expected: 3,
                found: 2
            }]
        );
        // The edge {0, 2} is not in the path.
        let entries = [(Edge::undirected(0, 2), Label::Int(1))];
        let stray = KeyedLabeling::new("w", Kind::Int, entries);
        let errors = Labeled::new(path(false))
            .with_edge_labeling(stray)
            .err()
            .unwrap();
        assert_eq!(errors.as_slice().len(), 1);
    }

    #[test]
    fn an_invalid_labeling_is_rejected_by_the_labeled_graph() {
        let bad_node = IntLabeling::new("2n", 0..3);
        assert_eq!(
            Labeled::new(path(false)).with_node_labeling(bad_node).err(),
            Some(Error::InvalidName("2n".to_string()).into())
        );
        // Every problem is reported, not only the first.
        let bad_edge = IntLabeling::new("2w", 3..3);
        let errors = Labeled::new(path(false))
            .with_edge_labeling(bad_edge)
            .err()
            .unwrap();
        assert_eq!(
            errors.as_slice(),
            [Error::InvalidName("2w".to_string()), Error::EmptyRange]
        );
    }

    #[test]
    fn the_context_decides_the_computed_labels_and_not_the_stored_ones() {
        let int = || IntLabeling::new("n", 0..1_000_000);
        let stored = DenseLabeling::new(
            "s",
            Kind::Int,
            [Label::Int(1), Label::Int(2), Label::Int(3)],
        );
        let base = Labeled::new(path(false))
            .with_node_labeling(int())
            .unwrap()
            .with_node_labeling(stored)
            .unwrap();
        let seeded = |seed| {
            base.clone()
                .with_context(Context::default().with_seed(seed))
        };
        let (a, b) = (seeded(1), seeded(2));
        assert_eq!(a.context().seed, 1);
        assert_ne!(a.node_label("n", 0), b.node_label("n", 0));
        assert_eq!(a.node_label("n", 0), seeded(1).node_label("n", 0));
        assert_eq!(a.node_label("s", 1), b.node_label("s", 1));
        // A reference to a context works too.
        let ctx = Context::default().with_seed(1);
        assert_eq!(
            base.with_context(&ctx).node_label("n", 2),
            a.node_label("n", 2)
        );
    }

    #[test]
    fn it_is_the_same_graph() {
        let g = Labeled::new(path(false));
        assert_eq!((g.order(), g.size(), g.is_directed()), (3, 2, false));
        assert!(g.has_node(2) && !g.has_node(3));
        assert!(g.has_edge(2, 1) && !g.has_edge(0, 2));
        assert!(g.edges().eq(path(false).edges()));
        assert_eq!(g.nodes(), 0..3);
        assert_eq!(g.graph().order(), 3);
    }

    #[test]
    fn node_labels_are_found_by_name() {
        let g = Labeled::new(path(false))
            .with_node_labeling(colors())
            .unwrap();
        assert_eq!(g.node_label("color", 1), Some(Label::Enum(1)));
        assert_eq!(g.node_label("color", 2), Some(Label::Enum(0)));
        assert_eq!(g.node_label("size", 1), None);
    }

    #[test]
    fn a_node_outside_the_graph_has_no_label() {
        // The labeling itself would answer for node 5 only if it had it; use a
        // computed one, which answers for any node.
        let int = IntLabeling::new("n", 0..10);
        let g = Labeled::new(path(false)).with_node_labeling(int).unwrap();
        assert!(g.node_label("n", 2).is_some());
        assert_eq!(g.node_label("n", 3), None);
        assert_eq!(g.node_label("n", u64::MAX), None);
    }

    #[test]
    fn edge_labels_use_the_directedness_of_the_graph() {
        let g = Labeled::new(path(false))
            .with_edge_labeling(weights(false))
            .unwrap();
        // Undirected: either order of the ends is the same edge.
        assert_eq!(g.edge_label("weight", 0, 1), Some(Label::Int(10)));
        assert_eq!(g.edge_label("weight", 1, 0), Some(Label::Int(10)));
        assert_eq!(g.edge_label("weight", 2, 1), Some(Label::Int(20)));

        let d = Labeled::new(path(true))
            .with_edge_labeling(weights(true))
            .unwrap();
        // Directed: the direction matters.
        assert_eq!(d.edge_label("weight", 0, 1), Some(Label::Int(10)));
        assert_eq!(d.edge_label("weight", 1, 0), None);
    }

    #[test]
    fn a_pair_that_is_not_an_edge_has_no_label() {
        let int = IntLabeling::new("w", 0..100);
        let g = Labeled::new(path(false)).with_edge_labeling(int).unwrap();
        assert!(g.edge_label("w", 0, 1).is_some());
        assert_eq!(g.edge_label("w", 0, 2), None);
        assert_eq!(g.edge_label("w", 0, 9), None);
        assert_eq!(g.edge_label("missing", 0, 1), None);
    }

    #[test]
    fn computed_labelings_work_on_a_computed_graph() {
        let graph = build_graph(Gnp::new(30, 0.3, 1), false);
        let int = IntLabeling::new("w", 1..10);
        let g = Labeled::new(graph).with_edge_labeling(int.clone()).unwrap();
        for e in g.edges() {
            let label = g.edge_label("w", e.u, e.v).unwrap();
            assert_eq!(
                label,
                crate::labels::Labeling::<Edge>::label(&int, &Context::default(), e).unwrap()
            );
            assert!(matches!(label, Label::Int(1..=9)));
        }
        // The same graph, labeled twice, gives the same labels.
        let again = Labeled::new(g.graph().clone())
            .with_edge_labeling(int)
            .unwrap();
        let e = g.edges().next().unwrap();
        assert_eq!(g.edge_label("w", e.u, e.v), again.edge_label("w", e.u, e.v));
    }

    #[test]
    fn nodes_and_edges_have_separate_name_spaces() {
        let g = Labeled::new(path(false))
            .with_node_labeling(BoolLabeling::new("x", 0.5))
            .unwrap()
            .with_edge_labeling(IntLabeling::new("x", 0..10))
            .unwrap();
        assert!(matches!(g.node_label("x", 0), Some(Label::Bool(_))));
        assert!(matches!(g.edge_label("x", 0, 1), Some(Label::Int(_))));
    }

    #[test]
    fn a_repeated_name_on_one_side_is_an_error() {
        let g = Labeled::new(path(false))
            .with_node_labeling(colors())
            .unwrap();
        assert_eq!(
            g.clone().with_node_labeling(colors()).err(),
            Some(Error::DuplicateName("color".to_string()).into())
        );
        let g = g.with_edge_labeling(weights(false)).unwrap();
        assert_eq!(
            g.with_edge_labeling(weights(false)).err(),
            Some(Error::DuplicateName("weight".to_string()).into())
        );
    }

    #[test]
    fn labelings_come_in_the_order_they_were_added() {
        let g = Labeled::new(path(false))
            .with_node_labeling(colors())
            .unwrap()
            .with_node_labeling(IntLabeling::new("age", 0..9))
            .unwrap()
            .with_edge_labeling(weights(false))
            .unwrap();
        let nodes: Vec<_> = g.node_labelings().map(|l| l.name().to_string()).collect();
        let edges: Vec<_> = g.edge_labelings().map(|l| l.name().to_string()).collect();
        assert_eq!(nodes, ["color", "age"]);
        assert_eq!(edges, ["weight"]);
    }
}
