use std::collections::HashMap;

/// Represents a Beancount option, which represent configuration points global to the file.
///
/// The general format of the Option directive is:
///     option Name Value
///
/// Example option:
///
///     option "title" "Ed’s Personal Ledger"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.e2iyrfrmstl
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct BcOption {
    /// Name of the option.
    name: String,

    /// Value of the option.
    val: String,
}

/// Represents a 'txn' (or '*' or '!') directive.  A transaction can be signified by any of those
/// three symbols, where 'txn' and '*' both indicated a completed transaction and '!' indicates an
/// incomplete transaction.  The lines that follow the first line of a transaction are for
/// “Postings.”  You can read more about those in the Posting struct.  (TODO: create doc link)
///
/// A transaction may have an optional “payee” and/or a “narration", where the payee is a string
/// that represents an external entity that is involved in the transaction.  A narration is a
/// description of the transaction that you write. It can be a comment about the context, the person
/// who accompanied you, some note about the product you bought... whatever you want it to be.
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.aki1rqfx1z8q
///
/// Both transactions and postings can have metadata.  The general form for a transaction is:
///
///     YYYY-MM-DD [txn|Flag] [[Payee] Narration]
///         [Key: Value]
///         ...
///         [Flag] Account       Amount [{Cost}] [@ Price]
///             [Key: Value]
///             ...
///
///     The two following transactions are equivalent.
///
///     2014-05-05 txn "Cafe Mogador" "Lamb tagine with wine"
///         Liabilities:CreditCard:CapitalOne         -37.45 USD
///         Expenses:Restaurant
///
///     2014-05-05 * "Cafe Mogador" "Lamb tagine with wine"
///         Liabilities:CreditCard:CapitalOne         -37.45 USD
///         Expenses:Restaurant
///
///     And this is an incomplete transaction with a payee of "Seaworld", a narration of "Tickets",
///     and a single posting.
///
///     2014-05-08 ! "Seaworld" "Tickets"
///         Liabilities:CreditCard:CapitalOne         -80.00 USD
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.up4dj751q84w
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Transaction {
    /// Whether or not a transaction is considered complete.
    ///
    /// *: Completed transaction, known amounts, “this looks correct.”
    /// !: Incomplete transaction, needs confirmation or revision, “this looks incorrect.”
    completed: bool,

    /// Postings belonging to this transaction.
    postings: Option<Vec<Posting>>,

    /// Payee of this transaction.
    payee: Option<String>,

    /// Narration of this transaction.
    narration: Option<String>,

    /// Tags associated with the transaction.
    tags: Option<Vec<Tag>>,

    /// Links associated with the transactions.
    links: Option<Vec<Link>>,

    /// Metadata attached to the transaction.
    meta: HashMap<String, String>,
}

/// Tag associated with a transaction directive.  In the below transaction, #berlin-trip-2014 is
/// the tag.  Tags allow you to mark a subset of transactions, enabling filtering on a tag(s) when
/// generating a report.
///
///     2014-04-23 * "Flight to Berlin" #berlin-trip-2014
///         Expenses:Flights              -1230.27 USD
///         Liabilities:CreditCard
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.oivvp5olom2v
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Tag {
    /// Name of the tag.
    name: String,
}

/// Links provide a way to link transactions together.  You may think of the link as a special kind
/// of tag that can be used to group together a set of financially related transactions over time.
/// For example you may use links to group together transactions that are each related with a
/// specific  invoice. This  allows to track payments (or write-offs) associated with the invoice:
///
///     2014-02-05 * "Invoice for January" ^invoice-pepe-studios-jan14
///         Income:Clients:PepeStudios           -8450.00 USD
///         Assets:AccountsReceivable
///
///     2014-02-20 * "Check deposit - payment from Pepe" ^invoice-pepe-studios-jan14
///         Assets:BofA:Checking                  8450.00 USD
///         Assets:AccountsReceivable
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.k4v5vkjukel7
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Link {
    /// Name of the link.
    name: String,
}

/// Represents a transaction posting.  Postings represent a single amount being deposited to or
/// withdrawn from an account.
///
/// Postings can have optionally have either a cost or a price.  A posting with a price might look
/// like this, where the price is the amount and commodity following the '@":
///
///     2012-11-03 * "Transfer to account in Canada"
///         Assets:MyBank:Checking            -400.00 USD @ 1.09 CAD
///         Assets:FR:SocGen:Checking          436.01 CAD
///
/// A posting with a cost is the same with the exception that it utilizes '@@'.
///
///     2012-11-03 * "Transfer to account in Canada"
///         Assets:MyBank:Checking            -400.00 USD @@ 436.01 CAD
///         Assets:FR:SocGen:Checking          436.01 CAD
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.mtqrwt24wnzs
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Posting {
    /// Account being posted to.
    account: Account,

    /// The amount being posted.
    amount: Option<f64>,

    /// The type of commodity being posted.
    comm: Option<String>,

    /// The price of this posting.
    price: Option<PostingPrice>,

    /// The cost of this posting.
    cost: Option<Cost>,
}

/// Represents a price that belongs to a posting.  Posting prices enable use cases where you want
/// to convert from one currency to another.
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.mtqrwt24wnzs
#[derive(Clone, Debug, Default, PartialEq, TypedBuilder)]
pub struct PostingPrice {
    /// The type of commodity for this price.
    comm: String,

    /// The amount of whatever commodity used in this price.
    amount: f64,
}

// TODO: Important Note. Amounts specified as either per-share or total prices or costs are always unsigned. It is an error to use a negative sign or a negative cost and Beancount will raise an error if you attempt to do so.

/// Represents a 'cost', which typically belongs to a posting.
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.mtqrwt24wnzs
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Cost {
    /// The type of commodity for this cost.
    comm: Option<String>,

    /// The amount of whatever commodity used in this cost.
    amount: Option<f64>,

    /// The date of the at-cost.
    date: Option<String>,

    /// The label of the cost.
    label: Option<String>,
}

/// Represents a 'open' directive.  This directive signifies the opening of an account.
///
///     1990-01-01 open Expenses:Restaurant
///
///     2014-05-01 open Liabilities:CreditCard:CapitalOne     USD
///
///     2015-02-01 open Assets:Cash:Pesos
///         description: "A shared account to contain our pocket of pesos"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.omdgvaikswd0
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Open {
    /// Date the account was opened.
    date: String,

    /// Account being opened.
    account: Account,

    /// Commodities allowed for the opened account.
    constraint_commodities: Option<String>,

    /// Metadata attached to the open.
    meta: HashMap<String, String>,
}

/// Represents a 'close' directive.  This directive signifies the closing of an account.
///
///     ; Closing credit card after fraud was detected.
///     2016-11-28 close Liabilities:CreditCard:CapitalOne
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.wf248e8stnac
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Close {
    /// Date the account was closed.
    date: String,

    /// Account being closed.
    account: Account,

    /// Metadata attatched to the close.
    meta: HashMap<String, String>,
}

/// Represents a 'commodity' directive.  This directive allows you to declare commodities,
/// although doing so is not required in order to use a commodity.  The purpose of this directive is
/// to attach commodity-specific metadata fields on it, so that it can be gathered by plugins later
/// on.
///
///     1867-01-01 commodity CAD
///         name: "Canadian Dollar"
///         asset-class: "cash"
///
///     2012-01-01 commodity HOOL
///         name: "Hooli Corporation Class C Shares"
///         asset-class: "stock"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.a3si01ejc035
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Commodity {
    /// Date the commodity was declared.
    date: String,

    /// Commodity name.
    name: String,

    /// Metadata attached to the commodity.
    meta: HashMap<String, String>,
}

/// Represents a 'balance' directive.  A balance directive is a way for you to input your statement
/// balance into the flow of transactions. It tells Beancount to verify that the number of units of
/// a particular commodity in some account should equal some expected value at some point in time
///
/// The general format of the Balance directive is:
///     YYYY-MM-DD balance Account  Amount
///
///     ; Check cash balances from wallet
///     2014-08-09 balance Assets:Cash     562.00 USD
///     2014-08-09 balance Assets:Cash     210.00 CAD
///     2014-08-09 balance Assets:Cash      60.00 EUR
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.l0pvgeniwvq8
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Balance {
    /// Date of the balance.
    date: String,

    /// Account to check the balance of.
    account: Account,

    /// Amount to balance.
    amount: f64,

    /// Type of commodity to balance.
    comm: String,
}

/// Represents a 'pad' directive.  A pad directive automatically inserts a transaction that will
/// make the subsequent balance assertion succeed, if it is needed. It inserts the difference needed
/// to fulfill that balance assertion. (What “rubber space” is in LaTeX, pad directives are to
/// balances in Beancount.)
///
/// The general format of the Pad directive is:
///     YYYY-MM-DD pad Account AccountPad
///
/// Pad example:
///     2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.aw8ic3d8k8rq
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Pad {
    /// Date of the pad.
    date: String,

    /// Account to pad into.
    pad_to_account: Account,

    /// Account to pad from.
    pad_from_account: Account,
}

/// Represents a 'note' directive.  A note directive is simply used to attach a dated comment to the
/// journal of a particular account.
///
/// The general format of the Note directive is:
///
///     YYYY-MM-DD note Account Description
///
/// Example of a note:
///
///     2013-11-03 note Liabilities:CreditCard "Called about fraudulent card."
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.c4cyaa6o6rqm
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Note {
    /// Date of the note.
    date: String,

    /// Account being noted.
    account: Account,

    /// Note description.
    desc: String,
}

/// Represents a 'document' directive.  A document directive can be used to attach an external file
/// to the journal of an account.
///
/// The general format of the Document directive is:
///
///     YYYY-MM-DD document Account  PathToDocument
///
/// Example of a document directive:
///
///     2013-11-03 document Liabilities:CreditCard "/home/joe/stmts/apr-2014.pdf"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.w1ins9jk4mq3
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Document {
    /// Date the document was linked.
    date: String,

    /// Account document is added to.
    account: Account,

    /// Filesystem path to the document.
    path: String,
}

/// Represents a 'price' directive.  Beancount sometimes creates an in-memory data store of prices
/// for each commodity, that is used for various reasons. In particular, it is used to report
/// unrealized gains on account holdings. Price directives can be used to provide data points for
/// this database. A price directive establishes the rate of exchange between one commodity
/// (the base commodity) and another (the quote commodity).
///
/// The general format of the price directive is:
///
///     YYYY-MM-DD price Commodity Price
///
/// This directive says: “The price of one unit of HOOL on July 9th, 2014 was 579.18 USD.”
///
///     2014-07-09 price HOOL  579.18 USD
///
/// Price entries for currency exchange rates work the same way:
///
///     2014-07-09 price USD  1.08 CAD
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.f78ym1dxtemh
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Price {
    /// Date of the price specification.
    date: String,

    /// The commodity being priced (a.k.a the base commodity).
    base_comm: String,

    /// The commodity being quoted (a.k.a the quote commodity).
    quote_comm: String,

    /// Value the base commodity is being quoted at.
    quote_val: f64,
}

/// Represents an 'event' directive.  Event directives are used to track the value of some variable
/// of your choice over time - for example, your location.
///
/// The general format of the event directive is:
///
///     YYYY-MM-DD event Name Value
///
/// An example event directive:
///
///     2014-07-09 event "location" "Paris, France"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.tm5fxddlik5x
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Event {
    /// Date the event occurred.
    date: String,

    /// Name of the event.
    name: String,

    /// New value of the event.
    val: String,
}

/// Represents a 'query' directive.  Query directives allow you to insert a query in the usual
/// stream of transactions.  It can be convenient to be able to associate SQL queries in a Beancount
/// file to be able to run these as a report automatically, and query directives enable this.
///
/// The general format of the query directive is:
///
///     YYYY-MM-DD query Name SqlContents
///
/// An example query directive:
///
///     2014-07-09 query "france-balances" "
///         SELECT account, sum(position) WHERE ‘trip-france-2014’ in tags"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.nw8fgvy4ub1w
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Query {
    /// Date on which the query should be run.
    date: String,

    /// Name of the query.
    name: String,

    /// Query contents.
    query: String,
}

/// Represents a 'custom' directive.  The long-term plan for Beancount is to allow plugins and
/// external clients to define their own directive types, to be declared and validated by the
/// Beancount input language parser. In the meantime, a generic directive is provided for clients
/// to prototype new features, e.g., budgeting.
///
/// The grammar for this directive is flexible:
///
///     YYYY-MM-DD custom TypeName Value1 ...
///
/// The first argument is a string and is intended to be unique to your directive. Think of this as
/// the type of your directive. Following it, you can put an arbitrary list of strings, dates,
/// booleans, amounts, and numbers.
///
/// Example custom directive:
///
///     2014-07-09 custom "budget" "..." TRUE 45.30 USD
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.20klpeqb6ajy
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Custom {
    /// Date associated with the custom directive.
    date: String,

    /// Custom directive name.
    name: String,

    /// Arbitrary number of custom directive arguments.
    args: Vec<String>,
}

/// Represents a 'plugin' directive.  In the Python version of Beancount, this would allow you to
/// specify an actual arbitrary Python program to programmatically transform directives as they are
/// parsed.  That is clearly not (easily) possible in this Rust implementation, but we will still
/// give you back any 'plugin' directives we found in the form of this struct.
///
/// The general format of the plugin directive is:
///
///     plugin ModuleName StringConfig
///
/// Example plugin directive:
///
///     plugin "beancount.plugins.module_name" "configuration data"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.lxgs9ewvbt8k
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Plugin {
    /// Full module name of the plugin.
    module: String,

    /// Configuration data to be passed to the plugin.
    config: String,
}

/// Represents an 'include' directive.  The include directive, as it sounds, includes another
/// Beancount file into the current one, allowing you to arbitrarily split up your ledger files.
///
/// The general format is:
///
///     include Filename
///
/// Example include directive:
///
///     include "path/to/include/file.beancount"
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.86lelow4097r
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Include {
    /// Fully qualified filename, including any necessary path segements.
    filename: String,
}

/// Allowed account types.
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.17ry42rqbuiu
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountType {
    Assets,
    Liabilities,
    Equity,
    Income,
    Expenses,
}

/// Represents an account.  Beancount accumulates commodities in accounts.  An account name is a
/// colon-separated list of capitalized words which begin with a letter, and whose first word must
/// be one of the five acceptable account types.
///
///     Assets:US:BofA:Checking
///     Liabilities:CA:RBC:CreditCard
///     Equity:Retained-Earnings
///     Income:US:Acme:Salary
///     Expenses:Food:Groceries
///
/// https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.17ry42rqbuiu
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Account {
    /// Type of the account.
    ty: AccountType,

    /// Optional parts of the account following the account type.
    parts: Option<Vec<String>>,
}