use std::borrow::Cow;
use std::collections::HashMap;

use bigdecimal::BigDecimal;

/// Metadata that can be attached to other Beancount information.
pub type Meta<'a> = HashMap<Cow<'a, str>, MetaValue<'a>>;

/// An enum of the valid values in a metadata map.
// TODO: Implement Display
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum MetaValue<'a> {
    Text(Cow<'a, str>),
    Account(super::account::Account<'a>),
    Date(super::Date<'a>),
    Currency(super::Currency<'a>),
    Tag(Tag<'a>),
    Bool(bool),
    Amount(super::amount::Amount<'a>),
    Number(BigDecimal),
}

/// Tag associated with a transaction directive.  Tags allow you to mark a subset of transactions,
/// enabling filtering on a tag(s) when generating a report.
///
/// In the below transaction, #berlin-trip-2014 is the tag:
///
/// ```text
///
/// 2014-04-23 * "Flight to Berlin" #berlin-trip-2014
///     Expenses:Flights              -1230.27 USD
///     Liabilities:CreditCard
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.oivvp5olom2v>
pub type Tag<'a> = Cow<'a, str>;

/// Links provide a way to link transactions together.  You may think of the link as a special kind
/// of tag that can be used to group together a set of financially related transactions over time.
///
/// For example, you may use links to group together transactions that are each related with a
/// specific  invoice. This  allows to track payments (or write-offs) associated with the invoice:
///
/// Some transactions that have links:
///
/// ```text
/// 2014-02-05 * "Invoice for January" ^invoice-pepe-studios-jan14
///     Income:Clients:PepeStudios           -8450.00 USD
///     Assets:AccountsReceivable
///
/// 2014-02-20 * "Check deposit - payment from Pepe" ^invoice-pepe-studios-jan14
///     Assets:BofA:Checking                  8450.00 USD
///     Assets:AccountsReceivable
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.k4v5vkjukel7>
pub type Link<'a> = Cow<'a, str>;
