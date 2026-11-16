//! [`KeyedLabeling`]: labels for some elements, found by key.

use std::fmt::Debug;

use super::super::labeling::{fits, name_error, variants_error};
use super::super::{Error, Errors, Kind, Label, Labeling, Variants};
use crate::{Context, Edge, Graph, Node};

/// A stored labeling for elements found by an ordered key, such as the edges of a graph: the keys
/// are kept sorted, with the labels in the same order, so a lookup is a binary search. It may cover
/// only some elements; a missing key has no label. Keys are compared as given, so an undirected
/// edge must always be spelled the same way.
///
/// The constructors do not check their parameters; [`validate`](Labeling::validate) reports
/// [`Error::InvalidName`] if the name is not an identifier, [`Error::MissingVariants`] for an
/// enumeration made with [`new`](KeyedLabeling::new), the error of [`Variants::validate`] for bad
/// variants, [`Error::DuplicateKey`] for each repeated key, and [`Error::BadKeyedLabel`] for the
/// first label that does not fit. Reading an invalid labeling never panics, but the labels are
/// unspecified.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyedLabeling<E> {
    name: String,
    kind: Kind,
    variants: Option<Variants>,
    keys: Vec<E>,
    labels: Vec<Label>,
}

impl<E: Ord> KeyedLabeling<E> {
    /// The labeling called `name` of the given `kind` that gives each key of `entries` its label,
    /// in any order. For an enumeration use [`enumeration`](KeyedLabeling::enumeration).
    pub fn new(
        name: impl Into<String>,
        kind: Kind,
        entries: impl IntoIterator<Item = (E, Label)>,
    ) -> KeyedLabeling<E> {
        KeyedLabeling::sorted(name.into(), kind, None, entries)
    }

    /// The labeling called `name` of an enumeration with the given `variants` that gives each key
    /// of `entries` its label, in any order.
    pub fn enumeration(
        name: impl Into<String>,
        variants: Variants,
        entries: impl IntoIterator<Item = (E, Label)>,
    ) -> KeyedLabeling<E> {
        KeyedLabeling::sorted(name.into(), Kind::Enum, Some(variants), entries)
    }

    /// The labeling with the entries sorted by key; a repeated key keeps its input order.
    fn sorted(
        name: String,
        kind: Kind,
        variants: Option<Variants>,
        entries: impl IntoIterator<Item = (E, Label)>,
    ) -> KeyedLabeling<E> {
        let mut entries: Vec<(E, Label)> = entries.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let (keys, labels) = entries.into_iter().unzip();
        KeyedLabeling {
            name,
            kind,
            variants,
            keys,
            labels,
        }
    }

    /// The number of elements that have a label.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether no element has a label.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The keys that have a label, sorted.
    pub fn keys(&self) -> &[E] {
        &self.keys
    }
}

impl<E: Ord + Debug> KeyedLabeling<E> {
    /// Every problem of the labeling, in the order of [`Labeling::validate`].
    fn problems(&self) -> Vec<Error> {
        let variants_problem = variants_error(&self.kind, self.variants.as_ref());
        let repeated = self
            .keys
            .windows(2)
            .enumerate()
            .filter(|(i, pair)| pair[0] == pair[1] && (*i == 0 || self.keys[i - 1] != pair[0]))
            .map(|(_, pair)| Error::DuplicateKey(format!("{:?}", pair[0])));
        // With bad variants every enumeration label misfits, which is the same cause.
        let misfit = match variants_problem {
            None => self
                .keys
                .iter()
                .zip(&self.labels)
                .find(|(_, label)| !fits(&self.kind, self.variants.as_ref(), label))
                .map(|(key, label)| Error::BadKeyedLabel {
                    key: format!("{key:?}"),
                    kind: format!("{:?}", self.kind),
                    label: format!("{label:?}"),
                }),
            Some(_) => None,
        };
        name_error(&self.name)
            .into_iter()
            .chain(variants_problem)
            .chain(repeated)
            .chain(misfit)
            .collect()
    }
}

/// Whether the graph has the node `key`.
fn has_node_key(graph: &dyn Graph, key: &Node) -> bool {
    graph.has_node(*key)
}

/// Whether the graph has the edge `key`, spelled the way the graph spells it: an undirected edge
/// with `u <= v`.
fn has_edge_key(graph: &dyn Graph, key: &Edge) -> bool {
    *key == Edge::new(graph.is_directed(), key.u, key.v) && graph.has_edge(key.u, key.v)
}

/// The `Labeling` of a keyed labeling over one kind of element, which resolves against the graph
/// with `$has_key`.
macro_rules! keyed_labeling {
    ($element:ty, $has_key:ident) => {
        impl Labeling<$element> for KeyedLabeling<$element> {
            fn name(&self) -> &str {
                &self.name
            }

            fn kind(&self) -> &Kind {
                &self.kind
            }

            fn variants(&self) -> Option<&Variants> {
                self.variants.as_ref()
            }

            fn label(&self, _ctx: &Context, element: $element) -> Option<Label> {
                let index = self.keys.binary_search(&element).ok()?;
                self.labels.get(index).cloned()
            }

            fn validate(&self) -> std::result::Result<(), Errors> {
                Errors::from_list(self.problems())
            }

            fn resolve(&self, graph: &dyn Graph) -> std::result::Result<(), Errors> {
                Errors::from_list(
                    self.keys
                        .iter()
                        .filter(|key| !$has_key(graph, key))
                        .map(|key| Error::UnknownElement(format!("{key:?}"))),
                )
            }
        }
    };
}

keyed_labeling!(Node, has_node_key);
keyed_labeling!(Edge, has_edge_key);

#[cfg(test)]
mod tests {
    use super::super::super::Kind;
    use super::*;
    use crate::Edge;
    use crate::graphs::computed::generators::Path;
    use crate::testing::build_graph;

    /// Every problem that `validate` finds, in order (none if the labeling is valid).
    fn label_problems<E>(l: &impl Labeling<E>) -> Vec<Error> {
        l.validate()
            .err()
            .map(|errors| errors.as_slice().to_vec())
            .unwrap_or_default()
    }

    fn e(u: u64, v: u64) -> Edge {
        Edge::directed(u, v)
    }

    fn weights() -> KeyedLabeling<Edge> {
        let entries = [
            (e(2, 3), Label::Int(23)),
            (e(0, 1), Label::Int(1)),
            (e(1, 2), Label::Int(12)),
        ];
        KeyedLabeling::new("w", Kind::Int, entries)
    }

    #[test]
    fn finds_labels_by_key_whatever_the_input_order() {
        let w = weights();
        assert_eq!((w.len(), w.is_empty()), (3, false));
        assert_eq!(w.label(&Context::default(), e(0, 1)), Some(Label::Int(1)));
        assert_eq!(w.label(&Context::default(), e(1, 2)), Some(Label::Int(12)));
        assert_eq!(w.label(&Context::default(), e(2, 3)), Some(Label::Int(23)));
        assert_eq!(w.name(), "w");
    }

    #[test]
    fn keys_are_sorted() {
        assert_eq!(weights().keys(), [e(0, 1), e(1, 2), e(2, 3)]);
    }

    #[test]
    fn a_missing_key_has_no_label() {
        let w = weights();
        assert_eq!(w.label(&Context::default(), e(0, 2)), None);
        assert_eq!(w.label(&Context::default(), e(9, 9)), None);
    }

    #[test]
    fn validate_reports_each_repeated_key_once() {
        let entries = [
            (e(1, 1), Label::Int(0)),
            (e(0, 0), Label::Int(0)),
            (e(1, 1), Label::Int(5)),
            (e(1, 1), Label::Int(6)),
            (e(2, 2), Label::Int(1)),
            (e(2, 2), Label::Int(2)),
        ];
        let l = KeyedLabeling::new("w", Kind::Int, entries);
        assert_eq!(
            label_problems(&l),
            vec![
                Error::DuplicateKey(format!("{:?}", e(1, 1))),
                Error::DuplicateKey(format!("{:?}", e(2, 2)))
            ]
        );
        assert_eq!(label_problems(&weights()), vec![]);
    }

    #[test]
    fn validate_names_the_key_of_the_first_bad_label() {
        let entries = [
            (5u64, Label::Int(0)),
            (1, Label::Bool(true)),
            (3, Label::Int(0)),
        ];
        let l = KeyedLabeling::new("w", Kind::Int, entries);
        assert!(matches!(
            label_problems(&l).as_slice(),
            [Error::BadKeyedLabel { key, .. }] if key == "1"
        ));
    }

    #[test]
    fn validate_reports_the_name_first() {
        let l = KeyedLabeling::new("2w", Kind::Int, [(1u64, Label::Bool(true))]);
        let problems = label_problems(&l);
        assert_eq!(problems[0], Error::InvalidName("2w".to_string()));
        assert!(matches!(problems[1], Error::BadKeyedLabel { .. }));
        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn an_enumeration_needs_its_variants() {
        let entries = || [(7u64, Label::Enum(1)), (3, Label::Enum(0))];
        let missing = KeyedLabeling::new("c", Kind::Enum, entries());
        assert_eq!(label_problems(&missing), vec![Error::MissingVariants]);
        let colors = || Variants::new(["red", "green"]);
        let l = KeyedLabeling::enumeration("c", colors(), entries());
        assert_eq!(label_problems(&l), vec![]);
        assert_eq!(
            Labeling::<Node>::format(&l, &Label::Enum(1)).unwrap(),
            "green"
        );
        let bad = KeyedLabeling::enumeration("c", colors(), [(1u64, Label::Enum(2))]);
        assert!(matches!(
            label_problems(&bad).as_slice(),
            [Error::BadKeyedLabel { key, .. }] if key == "1"
        ));
        let empty = KeyedLabeling::<u64>::enumeration("c", Variants::new(Vec::<String>::new()), []);
        assert_eq!(label_problems(&empty), vec![Error::NoVariants]);
    }

    #[test]
    fn labels_of_every_kind_round_trip() {
        let labels = [
            (Kind::Bool, Label::Bool(true)),
            (Kind::Float, Label::Float(0.25)),
            (Kind::String, Label::String("s".into())),
        ];
        for (kind, label) in labels {
            let l = KeyedLabeling::new("x", kind, [(7u64, label.clone())]);
            assert_eq!(l.label(&Context::default(), 7), Some(label));
        }
    }

    #[test]
    fn resolve_accepts_keys_that_the_graph_has() {
        let g = build_graph(Path::new(4), true);
        assert_eq!(weights().resolve(&g), Ok(()));
        let nodes = KeyedLabeling::new("n", Kind::Int, [(0u64, Label::Int(1)), (3, Label::Int(2))]);
        assert_eq!(nodes.resolve(&g), Ok(()));
    }

    #[test]
    fn resolve_reports_every_key_the_graph_lacks() {
        let g = build_graph(Path::new(3), true);
        let errors = weights().resolve(&g).unwrap_err();
        assert_eq!(
            errors.as_slice(),
            [Error::UnknownElement(format!("{:?}", e(2, 3)))]
        );
        let nodes = KeyedLabeling::new("n", Kind::Int, [(1u64, Label::Int(1)), (3, Label::Int(2))]);
        let errors = nodes.resolve(&g).unwrap_err();
        assert_eq!(errors.as_slice(), [Error::UnknownElement("3".to_string())]);
    }

    #[test]
    fn resolve_wants_an_undirected_edge_spelled_with_the_smaller_end_first() {
        let g = build_graph(Path::new(3), false);
        let l = KeyedLabeling::new(
            "w",
            Kind::Int,
            [
                (Edge::directed(2, 1), Label::Int(0)),
                (Edge::directed(0, 1), Label::Int(0)),
            ],
        );
        let errors = l.resolve(&g).unwrap_err();
        assert_eq!(
            errors.as_slice(),
            [Error::UnknownElement(format!("{:?}", e(2, 1)))]
        );
    }
}
