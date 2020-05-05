use typed_builder::TypedBuilder;

use super::account::Account;
use super::amount::IncompleteAmount;
use super::flags::Flag;
use super::position::CostSpec;
use super::Meta;

/// Represents a transaction posting.  Postings represent a single amount being deposited to or
/// withdrawn from an account.
///
/// Postings can have optionally have either a cost or a price.  A posting with a price might look
/// like this, where the price is the amount and commodity following the `@`:
///
/// ```text
/// 2012-11-03 * "Transfer to account in Canada"
///     Assets:MyBank:Checking            -400.00 USD @ 1.09 CAD
///     Assets:FR:SocGen:Checking          436.01 CAD
/// ```
///
/// A posting with a cost is the same with the exception that it utilizes `@@`.
///
/// ```text
/// 2012-11-03 * "Transfer to account in Canada"
///     Assets:MyBank:Checking            -400.00 USD @@ 436.01 CAD
///     Assets:FR:SocGen:Checking          436.01 CAD
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.mtqrwt24wnzs>
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Posting<'a> {
    /// Account being posted to.
    pub account: Account<'a>,

    /// The amount being posted.
    pub units: IncompleteAmount<'a>,

    /// The cost of this posting.
    #[builder(default)]
    pub cost: Option<CostSpec<'a>>,

    /// The price of this posting.
    #[builder(default)]
    pub price: Option<IncompleteAmount<'a>>,

    #[builder(default)]
    pub flag: Option<Flag<'a>>,

    #[builder(default)]
    pub meta: Meta<'a>,
}
