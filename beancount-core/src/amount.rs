use std::convert::TryFrom;

use rust_decimal::Decimal;
use typed_builder::TypedBuilder;

use super::Currency;

/// A number of units of a certain commodity.
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Amount<'a> {
    /// The value of the amount.
    pub num: Decimal,

    /// The commodity of the amount.
    pub currency: Currency<'a>,
}

/// An amount that may have missing units and/or commodity.
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct IncompleteAmount<'a> {
    /// The (optional) value of the amount.
    #[builder(default)]
    pub num: Option<Decimal>,

    /// The (optional) commodity of the amount.
    #[builder(default)]
    pub currency: Option<Currency<'a>>,
}

impl<'a> TryFrom<IncompleteAmount<'a>> for Amount<'a> {
    type Error = ();

    fn try_from(val: IncompleteAmount<'a>) -> Result<Self, Self::Error> {
        match val {
            IncompleteAmount {
                num: Some(num),
                currency: Some(currency),
            } => Ok(Amount { num, currency }),
            _ => Err(()),
        }
    }
}

impl<'a> From<Amount<'a>> for IncompleteAmount<'a> {
    fn from(val: Amount<'a>) -> Self {
        IncompleteAmount {
            num: Some(val.num),
            currency: Some(val.currency),
        }
    }
}
