use std::borrow::Cow;
use std::fmt;

/// A flag for a posting or transaction.
///
/// # Example
/// ```rust
/// use beancount_core::Flag;
/// assert_eq!(Flag::default(), Flag::Okay);
/// assert_eq!(Flag::from("*"), Flag::Okay);
/// assert_eq!(Flag::from("!"), Flag::Warning);
/// assert_eq!(Flag::from(":)"), Flag::Other(":)".into()));
/// ```
// TODO: Make sure that the variant Other("*") can't be created, since Other("*") != Okay
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Flag<'a> {
    Okay,
    Warning,
    Other(Cow<'a, str>),
}

impl Default for Flag<'_> {
    fn default() -> Self {
        Flag::Okay
    }
}

impl<'a> From<&'a str> for Flag<'a> {
    fn from(s: &'a str) -> Self {
        Cow::from(s).into()
    }
}

impl From<String> for Flag<'_> {
    fn from(s: String) -> Self {
        Cow::from(s).into()
    }
}

impl<'a> From<Cow<'a, str>> for Flag<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        match &*s {
            "*" | "txn" => Flag::Okay,
            "!" => Flag::Warning,
            _ => Flag::Other(s),
        }
    }
}

impl fmt::Display for Flag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Flag::Okay => write!(f, "*"),
            Flag::Warning => write!(f, "!"),
            Flag::Other(s) => write!(f, "{}", s),
        }
    }
}
