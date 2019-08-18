use std::borrow::Cow;

use rust_decimal::Decimal;
use typed_builder::TypedBuilder;

use super::amount::Amount;

#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Cost<'a> {
    pub number: Decimal,
    pub currency: Cow<'a, str>,
    pub date: Cow<'a, str>,
    pub label: Option<Cow<'a, str>>,
}

// TODO: Important Note. Amounts specified as either per-share or total prices or costs are always
// unsigned. It is an error to use a negative sign or a negative cost and Beancount will raise an
// error if you attempt to do so.

/// Represents a "cost", which typically belongs to a [Posting](struct.Posting.html).
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.mtqrwt24wnzs>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct CostSpec<'a> {
    pub number_per: Option<Decimal>,
    pub number_total: Option<Decimal>,
    /// The type of commodity for this cost.
    pub currency: Option<Cow<'a, str>>,
    /// The date of the at-cost.
    pub date: Option<Cow<'a, str>>,
    /// The label of the cost.
    pub label: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Position<'a> {
    pub units: Amount<'a>,
    pub cost: Option<Cost<'a>>,
}
