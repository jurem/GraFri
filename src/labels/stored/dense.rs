//! [`DenseLabeling`]: one label per index `0..len`, kept in memory.

use super::super::labeling::{misfit, name_error, variants_error};
use super::super::{Error, Errors, Kind, Label, Labeling, Variants};
use crate::{Context, Graph};

/// A stored labeling for elements numbered `0..len`, such as the nodes of a graph: the label of
/// element `i` is the `i`-th label. The element type is any `E: TryInto<usize>`; an element that
/// does not fit a `usize`, or is not below `len`, has no label.
///
/// The constructors do not check their parameters; [`validate`](Labeling::validate) reports
/// [`Error::InvalidName`] if the name is not an identifier, [`Error::MissingVariants`] for an
/// enumeration made with [`new`](DenseLabeling::new), the error of [`Variants::validate`] for bad
/// variants, and [`Error::BadLabel`] for the first label that does not fit. Reading an invalid
/// labeling never panics, but the labels are unspecified.
#[derive(Debug, Clone, PartialEq)]
pub struct DenseLabeling {
    name: String,
    kind: Kind,
    variants: Option<Variants>,
    labels: Vec<Label>,
}

impl DenseLabeling {
    /// The labeling called `name` of the given `kind` whose `i`-th label is the `i`-th of
    /// `labels`. For an enumeration use [`enumeration`](DenseLabeling::enumeration).
    pub fn new(
        name: impl Into<String>,
        kind: Kind,
        labels: impl IntoIterator<Item = Label>,
    ) -> DenseLabeling {
        DenseLabeling {
            name: name.into(),
            kind,
            variants: None,
            labels: labels.into_iter().collect(),
        }
    }

    /// The labeling called `name` of an enumeration with the given `variants`, whose `i`-th label
    /// is the `i`-th of `labels`.
    pub fn enumeration(
        name: impl Into<String>,
        variants: Variants,
        labels: impl IntoIterator<Item = Label>,
    ) -> DenseLabeling {
        DenseLabeling {
            name: name.into(),
            kind: Kind::Enum,
            variants: Some(variants),
            labels: labels.into_iter().collect(),
        }
    }

    /// The number of elements that have a label.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Whether no element has a label.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

impl<E: TryInto<usize>> Labeling<E> for DenseLabeling {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &Kind {
        &self.kind
    }

    fn variants(&self) -> Option<&Variants> {
        self.variants.as_ref()
    }

    fn label(&self, _ctx: &Context, element: E) -> Option<Label> {
        self.labels.get(element.try_into().ok()?).cloned()
    }

    fn validate(&self) -> std::result::Result<(), Errors> {
        let variants_problem = variants_error(&self.kind, self.variants.as_ref());
        // With bad variants every enumeration label misfits, which is the same cause.
        let misfit = match variants_problem {
            None => misfit(&self.kind, self.variants.as_ref(), &self.labels),
            Some(_) => None,
        };
        let problems: Vec<Error> = name_error(&self.name)
            .into_iter()
            .chain(variants_problem)
            .chain(misfit)
            .collect();
        Errors::from_list(problems)
    }

    fn resolve(&self, graph: &dyn Graph) -> std::result::Result<(), Errors> {
        if graph.order() == self.len() as u64 {
            return Ok(());
        }
        Err(Error::LabelCount {
            expected: graph.order(),
            found: self.len(),
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{Error, Kind};
    use super::*;
    use crate::testing::label_problems;

    fn weights() -> DenseLabeling {
        let labels = [Label::Int(10), Label::Int(20), Label::Int(30)];
        DenseLabeling::new("w", Kind::Int, labels)
    }

    #[test]
    fn label_is_the_label_at_that_index() {
        let w = weights();
        assert_eq!((w.len(), w.is_empty()), (3, false));
        assert_eq!(
            Labeling::<u64>::label(&w, &Context::default(), 0),
            Some(Label::Int(10))
        );
        assert_eq!(
            Labeling::<u64>::label(&w, &Context::default(), 2),
            Some(Label::Int(30))
        );
        assert_eq!(Labeling::<u64>::name(&w), "w");
    }

    #[test]
    fn out_of_range_elements_have_no_label() {
        let w = weights();
        assert_eq!(Labeling::<u64>::label(&w, &Context::default(), 3), None);
        assert_eq!(
            Labeling::<u64>::label(&w, &Context::default(), u64::MAX),
            None
        );
    }

    #[test]
    fn works_for_any_index_type() {
        let w = weights();
        assert_eq!(
            Labeling::<u32>::label(&w, &Context::default(), 1),
            Some(Label::Int(20))
        );
        assert_eq!(
            Labeling::<usize>::label(&w, &Context::default(), 1),
            Some(Label::Int(20))
        );
        assert_eq!(
            Labeling::<u8>::label(&w, &Context::default(), 1),
            Some(Label::Int(20))
        );
    }

    #[test]
    fn negative_indices_have_no_label() {
        let w = weights();
        assert_eq!(Labeling::<i64>::label(&w, &Context::default(), -1), None);
        assert_eq!(
            Labeling::<i64>::label(&w, &Context::default(), 1),
            Some(Label::Int(20))
        );
    }

    #[test]
    fn validate_reports_labels_the_kind_does_not_accept() {
        let l = DenseLabeling::new("w", Kind::Int, [Label::Int(1), Label::Bool(true)]);
        assert!(matches!(
            label_problems(&l).as_slice(),
            [Error::BadLabel { index: 1, .. }]
        ));
        assert_eq!(label_problems(&weights()), vec![]);
    }

    #[test]
    fn validate_reports_the_name_first() {
        let l = DenseLabeling::new("2w", Kind::Int, [Label::Bool(true)]);
        let problems = label_problems(&l);
        assert_eq!(problems[0], Error::InvalidName("2w".to_string()));
        assert!(matches!(problems[1], Error::BadLabel { index: 0, .. }));
        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn labels_of_every_kind_round_trip() {
        let labels = [
            (Kind::Bool, Label::Bool(true)),
            (Kind::Int, Label::Int(-5)),
            (Kind::Float, Label::Float(0.25)),
            (Kind::String, Label::String("s".into())),
        ];
        for (kind, label) in labels {
            let l = DenseLabeling::new("x", kind, [label.clone()]);
            assert_eq!(
                Labeling::<u64>::label(&l, &Context::default(), 0),
                Some(label)
            );
        }
    }

    #[test]
    fn an_enumeration_needs_its_variants() {
        let labels = [Label::Enum(0), Label::Enum(1)];
        let missing = DenseLabeling::new("c", Kind::Enum, labels.clone());
        assert_eq!(label_problems(&missing), vec![Error::MissingVariants]);
        let colors = || Variants::new(["red", "green"]);
        let l = DenseLabeling::enumeration("c", colors(), labels);
        assert_eq!(label_problems(&l), vec![]);
        assert_eq!(
            Labeling::<u64>::format(&l, &Label::Enum(1)).unwrap(),
            "green"
        );
        let bad = DenseLabeling::enumeration("c", colors(), [Label::Enum(2)]);
        assert!(matches!(
            label_problems(&bad).as_slice(),
            [Error::BadLabel { index: 0, .. }]
        ));
        let empty = DenseLabeling::enumeration("c", Variants::new(Vec::<String>::new()), []);
        assert_eq!(label_problems(&empty), vec![Error::NoVariants]);
    }

    #[test]
    fn resolve_wants_one_label_per_node() {
        use crate::graphs::computed::generators::Path;
        use crate::testing::build_graph;
        assert_eq!(
            Labeling::<u64>::resolve(&weights(), &build_graph(Path::new(3), false)),
            Ok(())
        );
        let errors =
            Labeling::<u64>::resolve(&weights(), &build_graph(Path::new(5), false)).unwrap_err();
        assert_eq!(
            errors.as_slice(),
            [Error::LabelCount {
                expected: 5,
                found: 3
            }]
        );
    }
}
