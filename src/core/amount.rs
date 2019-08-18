use std::borrow::Cow;

use rust_decimal::Decimal;
use typed_builder::TypedBuilder;

/// A number of units of a certain commodity.
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Amount<'a> {
    /// The (optional) value of the amount.
    pub num: Decimal,

    /// The commodity of the amount.
    pub commodity: Cow<'a, str>,
}

/// An amount that may have missing units and/or commidity.
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct IncompleteAmount<'a> {
    /// The (optional) value of the amount.
    #[builder(default)]
    pub num: Option<Decimal>,

    /// The commodity of the amount.
    #[builder(default)]
    pub commodity: Option<Cow<'a, str>>,
}
