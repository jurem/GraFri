//! [`Errors`]: a non-empty list of errors.

use std::fmt;

/// The errors found by a check: at least one, in order. Generic over the error type; each area has
/// an alias, [`graphs::Errors`](crate::Errors) and [`labels::Errors`](crate::labels::Errors).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Errors<E>(Vec<E>);

impl<E> Errors<E> {
    /// Joins the results of several checks: `Ok` if all are, otherwise all their errors, in order.
    pub fn collect(
        results: impl IntoIterator<Item = Result<(), Errors<E>>>,
    ) -> Result<(), Errors<E>> {
        Errors::from_list(
            results
                .into_iter()
                .filter_map(Result::err)
                .flat_map(|found| found.0),
        )
    }

    /// `Ok` if there are no errors, otherwise all of them, in order.
    pub fn from_list(errors: impl IntoIterator<Item = E>) -> Result<(), Errors<E>> {
        let errors: Vec<E> = errors.into_iter().collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Errors(errors))
        }
    }

    /// The errors, in order. Never empty.
    pub fn as_slice(&self) -> &[E] {
        &self.0
    }
}

/// A single error is a list of one.
impl<E> From<E> for Errors<E> {
    fn from(error: E) -> Errors<E> {
        Errors(vec![error])
    }
}

impl<E: fmt::Display> fmt::Display for Errors<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid: ")?;
        for (i, e) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for Errors<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_joins_the_errors_in_order() {
        assert_eq!(Errors::<u8>::collect([Ok(()), Ok(())]), Ok(()));
        assert_eq!(Errors::<u8>::collect([]), Ok(()));
        let ab = Errors::from_list([1, 2]).unwrap_err();
        let joined = Errors::collect([Err(ab), Ok(()), Err(3.into())]);
        assert_eq!(joined.unwrap_err().as_slice(), [1, 2, 3]);
    }

    #[test]
    fn from_list_is_ok_only_for_an_empty_list() {
        assert_eq!(Errors::<u8>::from_list([]), Ok(()));
        assert_eq!(Errors::from_list([7]).unwrap_err().as_slice(), [7]);
    }

    #[test]
    fn display_lists_every_error() {
        let errors = Errors::from_list(["a is wrong", "b is wrong"]).unwrap_err();
        assert_eq!(errors.to_string(), "invalid: a is wrong; b is wrong");
    }
}
