use std::borrow::Cow;

use typed_builder::TypedBuilder;

pub use account::Account;
pub use account_types::AccountType;
pub use amount::{Amount, IncompleteAmount};
pub use date::Date;
pub use directives::*;
pub use flags::Flag;
pub use position::CostSpec;
pub use posting::Posting;

pub mod account;
pub mod account_types;
pub mod amount;
mod date;
pub mod directives;
pub mod flags;
pub mod metadata;
pub mod position;
pub mod posting;

/// Represents the complete ledger consisting of a number of directives.
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Ledger<'a> {
    pub directives: Vec<Directive<'a>>,
}

pub type Currency<'a> = Cow<'a, str>;
