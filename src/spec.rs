//! [`Spec`]: the root of a graph: its directedness, seed, expression and labels.

use std::fmt;
use std::sync::Arc;

use crate::Context;
use crate::error::Result;
use crate::graphs::computed::{Expr, GraphOf, Term};
use crate::graphs::{Edge, Node};
use crate::graphs::{Error as GraphError, Errors};
use crate::labeled::{EdgeLabeling, Labeled, NodeLabeling};
use crate::labels::{self, Labeling};

/// Everything that determines a graph: the directedness, the master seed, an expression and the
/// labelings. The same spec gives the same graph on every platform. It corresponds to a graph block
/// of the spec language.
///
/// [`Spec::new`] is an undirected graph with seed 0, no expression and no labels. The `with_*`
/// methods, [`directed`](Spec::directed) and [`undirected`](Spec::undirected) change it in any
/// order, each returning a changed copy. [`build`](Spec::build) reports
/// [`MissingExpression`](crate::graphs::Error::MissingExpression) if there is no expression.
///
/// Labelings are independent of the structure. A computed labeling has a stream, by default derived
/// from its name, and the master seed of the spec decides its labels, so
/// [`with_seed`](Spec::with_seed) re-samples the structure and the labels together.
#[derive(Clone)]
pub struct Spec {
    directed: bool,
    seed: u64,
    term: Option<Arc<dyn Term>>,
    node_labelings: Vec<Arc<dyn NodeLabeling>>,
    edge_labelings: Vec<Arc<dyn EdgeLabeling>>,
}

/// The same as [`Spec::new`].
impl Default for Spec {
    fn default() -> Spec {
        Spec::new()
    }
}

impl Spec {
    /// An undirected spec with seed 0, no expression and no labels.
    pub fn new() -> Spec {
        Spec {
            directed: false,
            seed: 0,
            term: None,
            node_labelings: Vec::new(),
            edge_labelings: Vec::new(),
        }
    }

    /// The same spec for a directed graph.
    pub fn directed(self) -> Spec {
        Spec {
            directed: true,
            ..self
        }
    }

    /// The same spec for an undirected graph. This is the default.
    pub fn undirected(self) -> Spec {
        Spec {
            directed: false,
            ..self
        }
    }

    /// The same spec with another master seed.
    pub fn with_seed(self, seed: u64) -> Spec {
        Spec { seed, ..self }
    }

    /// The same spec with the graph that `term` describes, replacing any earlier one. `term` is a
    /// [`Expr`] (or a reference to one), or any raw term.
    pub fn with_expr(self, term: impl Into<Expr>) -> Spec {
        Spec {
            term: Some(term.into().term()),
            ..self
        }
    }

    /// The same spec with one more node labeling. Nothing is checked here: an invalid labeling
    /// or a repeated name is reported by [`build_labeled`](Spec::build_labeled).
    pub fn with_node_labeling(mut self, labeling: impl NodeLabeling + 'static) -> Spec {
        self.node_labelings.push(Arc::new(labeling));
        self
    }

    /// The same spec with one more edge labeling. Nothing is checked here: an invalid labeling
    /// or a repeated name is reported by [`build_labeled`](Spec::build_labeled).
    pub fn with_edge_labeling(mut self, labeling: impl EdgeLabeling + 'static) -> Spec {
        self.edge_labelings.push(Arc::new(labeling));
        self
    }

    /// The context that the expression is read under.
    pub fn context(&self) -> Context {
        Context::new(self.directed).with_seed(self.seed)
    }

    /// The graph without the labels, or every error that [`Term::check`] finds in the expression,
    /// or [`MissingExpression`](crate::graphs::Error::MissingExpression) if there is none.
    pub fn build(&self) -> std::result::Result<GraphOf, Errors> {
        let term = self.term.as_ref().ok_or(GraphError::MissingExpression)?;
        term.check(&self.context())?;
        Ok(GraphOf::new(Arc::clone(term), self.context()))
    }

    /// The graph with its labels.
    ///
    /// # Errors
    /// In three phases: [`Error::Check`](crate::Error::Check) with the errors of
    /// [`build`](Spec::build); then, if there are none, [`Error::Labels`](crate::Error::Labels)
    /// with the problems of all the labelings ([`Labeling::validate`], and each repeated name),
    /// node labelings first; then, if there are none, [`Error::Labels`](crate::Error::Labels) with
    /// the ways in which the labelings do not fit the graph ([`Labeling::resolve`]), in the same
    /// order.
    pub fn build_labeled(&self) -> Result<Labeled<GraphOf>> {
        let graph = self.build()?;
        let mut problems = labeling_problems::<dyn NodeLabeling, Node>(&self.node_labelings);
        problems.extend(labeling_problems::<dyn EdgeLabeling, Edge>(
            &self.edge_labelings,
        ));
        labels::Errors::from_list(problems)?;
        let mut problems = fit_problems::<dyn NodeLabeling, Node>(&self.node_labelings, &graph);
        problems.extend(fit_problems::<dyn EdgeLabeling, Edge>(
            &self.edge_labelings,
            &graph,
        ));
        labels::Errors::from_list(problems)?;
        Ok(Labeled::from_checked(
            graph,
            self.context(),
            self.node_labelings.clone(),
            self.edge_labelings.clone(),
        ))
    }
}

/// The problems of some labelings, in order: each labeling's own (see [`Labeling::validate`]), and
/// a [`DuplicateName`](labels::Error::DuplicateName) for each labeling whose name an earlier one
/// has.
fn labeling_problems<L: ?Sized + Labeling<E>, E>(labelings: &[Arc<L>]) -> Vec<labels::Error> {
    let mut problems = Vec::new();
    let mut names: Vec<&str> = Vec::new();
    for labeling in labelings {
        if let Err(errors) = labeling.validate() {
            problems.extend(errors.as_slice().iter().cloned());
        }
        let name = labeling.name();
        if names.contains(&name) {
            problems.push(labels::Error::DuplicateName(name.to_string()));
        } else {
            names.push(name);
        }
    }
    problems
}

/// The ways in which some labelings do not fit `graph`, in order (see [`Labeling::resolve`]).
fn fit_problems<L: ?Sized + Labeling<E>, E>(
    labelings: &[Arc<L>],
    graph: &GraphOf,
) -> Vec<labels::Error> {
    labelings
        .iter()
        .filter_map(|labeling| labeling.resolve(graph).err())
        .flat_map(|errors| errors.as_slice().to_vec())
        .collect()
}

/// Shown as the directedness, the seed, the expression and the names of the labelings.
impl fmt::Debug for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = |infos: Vec<&str>| format!("[{}]", infos.join(", "));
        let nodes = names(self.node_labelings.iter().map(|l| l.name()).collect());
        let edges = names(self.edge_labelings.iter().map(|l| l.name()).collect());
        let term = match &self.term {
            Some(term) => format!("{term:?}"),
            None => "no expression".to_string(),
        };
        f.debug_tuple("Spec")
            .field(&format_args!(
                "{}",
                if self.directed {
                    "directed"
                } else {
                    "undirected"
                }
            ))
            .field(&format_args!("seed {}", self.seed))
            .field(&format_args!("{term}"))
            .field(&format_args!("node_labelings {nodes}"))
            .field(&format_args!("edge_labelings {edges}"))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::computed::generators::{Complete, Gnp, Path};
    use crate::graphs::{Error as GraphError, Graph};
    use crate::labeled::{EdgeLabels, NodeLabels};
    use crate::labels::{
        self, BoolLabeling, ChoiceLabeling, DenseLabeling, FloatLabeling, IntLabeling, Kind, Label,
    };
    use crate::testing::build_graph_in;

    fn gnp_spec() -> Spec {
        Spec::new().with_expr(Expr::new(Gnp::new(30, 0.3, 1)) | Path::new(30))
    }

    #[test]
    fn the_same_expression_serves_both_directednesses() {
        let spec = Spec::new().with_expr(Complete::new(4));
        let u = spec.clone().undirected().build().unwrap();
        let d = spec.directed().build().unwrap();
        assert_eq!((u.is_directed(), u.size()), (false, 6));
        assert_eq!((d.is_directed(), d.size()), (true, 12));
    }

    #[test]
    fn it_gives_the_graph_of_the_same_builder_and_context() {
        let expr = Expr::new(Gnp::new(30, 0.3, 1)) - Path::new(30);
        for directed in [false, true] {
            let by_builder = build_graph_in(expr.term(), Context::new(directed).with_seed(7));
            let spec = Spec::new().with_seed(7).with_expr(&expr);
            let by_spec = if directed { spec.directed() } else { spec }
                .build()
                .unwrap();
            assert!(by_builder.edges().eq(by_spec.edges()));
        }
    }

    #[test]
    fn the_seed_decides_the_random_leaves() {
        let (a, b, c) = (
            gnp_spec().with_seed(1),
            gnp_spec().with_seed(1),
            gnp_spec().with_seed(2),
        );
        let (a, b, c) = (a.build().unwrap(), b.build().unwrap(), c.build().unwrap());
        assert!(a.edges().eq(b.edges()));
        assert!(!a.edges().eq(c.edges()));
    }

    #[test]
    fn build_reports_the_errors_of_the_expression() {
        let spec = Spec::new().with_expr(Expr::new(Gnp::new(5, 2.0, 1)) | Complete::new(3));
        let errors = spec.build().unwrap_err();
        assert!(matches!(
            errors.as_slice(),
            [GraphError::InvalidParam { name: "p", .. }]
        ));
        assert!(matches!(
            spec.build_labeled().err(),
            Some(crate::Error::Check(_))
        ));
    }

    #[test]
    fn labelings_are_attached_to_the_graph_in_order() {
        let spec = Spec::new()
            .with_seed(3)
            .with_expr(Complete::new(5))
            .with_node_labeling(ChoiceLabeling::new("color", ["red", "green"]))
            .with_edge_labeling(IntLabeling::new("weight", 1..10));
        let g = spec.build_labeled().unwrap();
        let direct = ChoiceLabeling::new("color", ["red", "green"]);
        for v in g.nodes() {
            assert_eq!(
                g.node_label("color", v),
                Labeling::<u64>::label(&direct, &Context::default().with_seed(3), v)
            );
        }
        assert!(matches!(
            g.edge_label("weight", 0, 1),
            Some(Label::Int(1..=9))
        ));
        assert_eq!(g.edge_label("weight", 0, 0), None);
        assert_eq!(g.node_labelings().count(), 1);
        assert_eq!(g.edge_labelings().count(), 1);
        assert_eq!(g.graph().context().seed, 3);
    }

    #[test]
    fn the_master_seed_of_the_spec_decides_the_labels_too() {
        let spec = Spec::new()
            .with_expr(Complete::new(40))
            .with_node_labeling(ChoiceLabeling::new("color", ["red", "green", "blue"]))
            .with_edge_labeling(IntLabeling::new("weight", 0..1000));
        let labels = |seed| {
            let g = spec.clone().with_seed(seed).build_labeled().unwrap();
            let colors: Vec<_> = g.nodes().map(|v| g.node_label("color", v)).collect();
            let weights: Vec<_> = g
                .edges()
                .map(|e| g.edge_label("weight", e.u, e.v))
                .collect();
            (colors, weights)
        };
        // The same seed gives the same labels, another seed gives other ones.
        assert_eq!(labels(1), labels(1));
        let (colors, weights) = labels(1);
        let (other_colors, other_weights) = labels(2);
        assert_ne!(colors, other_colors);
        assert_ne!(weights, other_weights);
        // Two labelings with different names are independent of each other.
        let both = spec
            .clone()
            .with_node_labeling(ChoiceLabeling::new("shade", ["red", "green", "blue"]))
            .build_labeled()
            .unwrap();
        let shades: Vec<_> = both.nodes().map(|v| both.node_label("shade", v)).collect();
        assert_ne!(colors, shades);
    }

    /// The problems that `build_labeled` reports for a labeling error, or none.
    fn label_problems(spec: &Spec) -> Vec<labels::Error> {
        match spec.build_labeled().err() {
            Some(crate::Error::Labels(errors)) => errors.as_slice().to_vec(),
            Some(other) => panic!("not a labeling error: {other:?}"),
            None => Vec::new(),
        }
    }

    #[test]
    fn a_repeated_name_is_an_error_on_each_side_reported_at_build() {
        let color = || ChoiceLabeling::new("color", ["a", "b"]);
        let weight = || IntLabeling::new("weight", 0..5);
        let spec = Spec::new().with_expr(Path::new(3));
        let duplicate = |name: &str| vec![labels::Error::DuplicateName(name.to_string())];
        // Nothing is reported while the spec is made.
        let with_color = spec.with_node_labeling(color());
        assert_eq!(
            label_problems(&with_color.clone().with_node_labeling(color())),
            duplicate("color")
        );
        // Nodes and edges have separate name spaces.
        let both = with_color.with_edge_labeling(weight());
        assert_eq!(
            label_problems(&both.clone().with_edge_labeling(weight())),
            duplicate("weight")
        );
        assert!(both.with_edge_labeling(color()).build_labeled().is_ok());
    }

    #[test]
    fn a_labeling_that_does_not_fit_is_reported_after_the_invalid_ones() {
        let short = || DenseLabeling::new("s", Kind::Int, [Label::Int(1)]);
        let fit = |name: &str| {
            let labels = (0..3).map(Label::Int);
            DenseLabeling::new(name, Kind::Int, labels)
        };
        let spec = Spec::new()
            .with_expr(Path::new(3))
            .with_node_labeling(fit("a"))
            .with_node_labeling(short());
        assert!(spec.build().is_ok());
        assert_eq!(
            label_problems(&spec),
            vec![labels::Error::LabelCount {
                expected: 3,
                found: 1
            }]
        );
        // An invalid labeling is reported alone, before the fit is looked at.
        let both = spec.with_node_labeling(IntLabeling::new("2n", 0..5));
        assert_eq!(
            label_problems(&both),
            vec![labels::Error::InvalidName("2n".to_string())]
        );
    }

    #[test]
    fn an_invalid_labeling_is_reported_by_build_labeled_and_not_by_build() {
        let spec = Spec::new()
            .with_expr(Path::new(3))
            .with_node_labeling(IntLabeling::new("2n", 0..5));
        assert!(spec.build().is_ok());
        assert_eq!(
            label_problems(&spec),
            vec![labels::Error::InvalidName("2n".to_string())]
        );
        let spec = Spec::new()
            .with_expr(Path::new(3))
            .with_edge_labeling(ChoiceLabeling::new("c", Vec::<String>::new()));
        assert_eq!(label_problems(&spec), vec![labels::Error::NoVariants]);
        // The expression is reported first, and alone.
        let both = Spec::new()
            .with_expr(Gnp::new(3, 2.0, 1))
            .with_node_labeling(IntLabeling::new("2n", 0..5));
        assert!(matches!(
            both.build_labeled().err(),
            Some(crate::Error::Check(_))
        ));
    }

    #[test]
    fn every_problem_of_every_labeling_is_reported_at_once() {
        // Two labelings with two problems each, a repeated name, and an edge labeling.
        let spec = Spec::new()
            .with_expr(Path::new(3))
            .with_node_labeling(IntLabeling::new("2n", 3..3))
            .with_node_labeling(BoolLabeling::new("ok", 0.5))
            .with_node_labeling(BoolLabeling::new("ok", 2.0))
            .with_edge_labeling(FloatLabeling::new("w", 1.0..0.0));
        assert_eq!(
            label_problems(&spec),
            vec![
                labels::Error::InvalidName("2n".to_string()),
                labels::Error::EmptyRange,
                labels::Error::InvalidProbability,
                labels::Error::DuplicateName("ok".to_string()),
                labels::Error::InvalidFloatRange,
            ]
        );
        // They are in the message too.
        let message = spec.build_labeled().err().unwrap().to_string();
        assert!(
            message.starts_with("invalid: `2n` is not a label name"),
            "{message}"
        );
        assert_eq!(message.matches("; ").count(), 4);
    }

    #[test]
    fn building_without_an_expression_is_an_error() {
        let spec = Spec::new().directed().with_seed(4);
        assert_eq!(
            spec.build().unwrap_err().as_slice(),
            [GraphError::MissingExpression]
        );
        assert!(matches!(
            spec.build_labeled().err(),
            Some(crate::Error::Check(_))
        ));
        assert_eq!(
            GraphError::MissingExpression.to_string(),
            "the spec has no expression"
        );
        // Giving one fixes it, and the other settings are kept.
        let g = spec.with_expr(Path::new(3)).build().unwrap();
        assert_eq!((g.is_directed(), g.context().seed, g.size()), (true, 4, 2));
    }

    #[test]
    fn with_expr_replaces_the_expression_and_keeps_the_rest() {
        let color = ChoiceLabeling::new("color", ["a", "b"]);
        let base = Spec::new().with_seed(8).with_node_labeling(color);
        let path = base
            .clone()
            .with_expr(Path::new(4))
            .build_labeled()
            .unwrap();
        let complete = base
            .clone()
            .with_expr(Path::new(2))
            .with_expr(Complete::new(4))
            .build_labeled()
            .unwrap();
        assert_eq!((path.order(), path.size()), (4, 3));
        assert_eq!((complete.order(), complete.size()), (4, 6));
        assert_eq!(path.node_labelings().count(), 1);
        assert_eq!(complete.node_labelings().count(), 1);
        assert_eq!(complete.graph().context().seed, 8);
        assert_eq!(
            Spec::default()
                .with_expr(Path::new(2))
                .build()
                .unwrap()
                .size(),
            1
        );
    }
}
