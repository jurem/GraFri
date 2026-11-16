//! The crate-level error: one variant for each area.

use std::fmt;

use crate::graphs;
use crate::labels;

/// The errors of the library. Each variant wraps the error of one area and converts from it, so `?`
/// works across the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A graph or one of its parameters is invalid.
    Graph(graphs::Error),
    /// A label, label name or enumeration is invalid.
    Label(labels::Error),
    /// A term failed its check.
    Check(graphs::Errors),
    /// One or more labelings failed their check, or two have the same name.
    Labels(labels::Errors),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Graph(e) => write!(f, "{e}"),
            Error::Label(e) => write!(f, "{e}"),
            Error::Check(e) => write!(f, "{e}"),
            Error::Labels(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Graph(e) => Some(e),
            Error::Label(e) => Some(e),
            Error::Check(e) => Some(e),
            Error::Labels(e) => Some(e),
        }
    }
}

impl From<graphs::Error> for Error {
    fn from(e: graphs::Error) -> Error {
        Error::Graph(e)
    }
}

impl From<graphs::Errors> for Error {
    fn from(e: graphs::Errors) -> Error {
        Error::Check(e)
    }
}

impl From<labels::Errors> for Error {
    fn from(e: labels::Errors) -> Error {
        Error::Labels(e)
    }
}

impl From<labels::Error> for Error {
    fn from(e: labels::Error) -> Error {
        Error::Label(e)
    }
}

/// Result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn an_area_error_converts_and_keeps_its_message() {
        let inner = labels::Error::NoVariants;
        let error: Error = inner.clone().into();
        assert_eq!(error, Error::Label(inner.clone()));
        assert_eq!(error.to_string(), inner.to_string());
        assert!(error.source().is_some());

        let inner = graphs::Error::InvalidParam {
            name: "p",
            message: "too big".to_string(),
        };
        let error: Error = inner.clone().into();
        assert_eq!(error, Error::Graph(inner.clone()));
        assert_eq!(error.to_string(), inner.to_string());
        assert!(error.source().is_some());
    }

    #[test]
    fn a_list_of_label_errors_converts_and_keeps_its_message() {
        let errors = labels::Errors::from_list([
            labels::Error::NoVariants,
            labels::Error::InvalidName("2c".to_string()),
        ])
        .unwrap_err();
        let error: Error = errors.clone().into();
        assert_eq!(error, Error::Labels(errors.clone()));
        assert_eq!(error.to_string(), errors.to_string());
        assert!(error.source().is_some());
    }

    #[test]
    fn question_mark_converts_each_area() {
        fn label() -> Result<()> {
            Ok(labels::Variants::new(Vec::<String>::new()).validate()?)
        }
        fn graph() -> Result<graphs::stored::EdgeList> {
            Ok(graphs::stored::EdgeList::new(2, false, [(0, 5)])?)
        }
        assert_eq!(label(), Err(Error::Label(labels::Error::NoVariants)));
        assert_eq!(
            graph(),
            Err(Error::Check(
                graphs::Error::EdgeOutOfRange {
                    edge: graphs::Edge::directed(0, 5),
                    order: 2
                }
                .into()
            ))
        );
    }
}
