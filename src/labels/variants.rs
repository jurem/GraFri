//! [`Variants`]: the variant names of an enumeration.

use std::sync::Arc;

use super::{Error, Result};

/// The variant names of an enumeration, in order: the label `i` is the `i`-th name. A valid one has
/// at least one name, none empty, all distinct, at most `u32::MAX`; [`Variants::validate`] checks
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variants(Arc<[String]>);

impl Variants {
    /// The variants with the given names, in order. The names are not checked.
    pub fn new<S: Into<String>>(names: impl IntoIterator<Item = S>) -> Variants {
        Variants(names.into_iter().map(Into::into).collect())
    }

    /// Checks the names.
    ///
    /// # Errors
    /// [`Error::NoVariants`], [`Error::TooManyVariants`] (more than `u32::MAX`),
    /// [`Error::EmptyVariant`] or [`Error::RepeatedVariant`], the first that applies.
    pub fn validate(&self) -> Result<()> {
        let names = &self.0;
        if names.is_empty() {
            return Err(Error::NoVariants);
        }
        if u32::try_from(names.len()).is_err() {
            return Err(Error::TooManyVariants);
        }
        if names.iter().any(String::is_empty) {
            return Err(Error::EmptyVariant);
        }
        for (i, name) in names.iter().enumerate() {
            if names[..i].contains(name) {
                return Err(Error::RepeatedVariant(name.clone()));
            }
        }
        Ok(())
    }

    /// The names, in order.
    pub fn names(&self) -> &[String] {
        &self.0
    }

    /// The name of the variant `index`, or `None` if there is no such variant.
    pub fn get(&self, index: u32) -> Option<&str> {
        self.0.get(index as usize).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validated(names: &[&str]) -> Result<()> {
        Variants::new(names.iter().copied()).validate()
    }

    #[test]
    fn validate_rejects_bad_names() {
        assert_eq!(validated(&[]), Err(Error::NoVariants));
        assert_eq!(validated(&["a", ""]), Err(Error::EmptyVariant));
        assert_eq!(
            validated(&["a", "b", "a"]),
            Err(Error::RepeatedVariant("a".to_string()))
        );
        assert!(validated(&["only"]).is_ok());
    }
}
