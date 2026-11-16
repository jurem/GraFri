//! The largest order of a term.

use crate::graphs::{Error, Errors};

/// The largest order of a term. A graph of this order has fewer than `2^64` edges or arcs,
/// loops included, so no size, count or node number overflows.
pub const MAX_ORDER: u64 = u32::MAX as u64;

/// Ok if `order` is at most [`MAX_ORDER`], else an error on the parameter(s) `name`.
pub(crate) fn check_order(name: &'static str, order: u128) -> Result<(), Errors> {
    if order <= u128::from(MAX_ORDER) {
        return Ok(());
    }
    Err(Error::InvalidParam {
        name,
        message: format!("the order must be at most {MAX_ORDER}, got {order}"),
    }
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_square_of_the_largest_order_fits_in_64_bits() {
        let max = u128::from(u64::MAX);
        let n = u128::from(MAX_ORDER);
        assert!(n * n <= max && (n + 1) * (n + 1) > max);
    }

    #[test]
    fn the_largest_order_is_accepted_and_one_more_is_not() {
        assert_eq!(check_order("order", MAX_ORDER.into()), Ok(()));
        let errors = check_order("order", u128::from(MAX_ORDER) + 1).unwrap_err();
        assert!(matches!(
            errors.as_slice(),
            [Error::InvalidParam { name: "order", .. }]
        ));
    }
}
