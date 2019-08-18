use std::borrow::Cow;

use rust_decimal::Decimal;
use typed_builder::TypedBuilder;

/// A number of units of a certain commodity.
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Amount<'a> {
    /// The (optional) value of the amount.
    pub num: Option<Decimal>,

    /// The commodity of the amount.
    pub commodity: Cow<'a, str>,
}
