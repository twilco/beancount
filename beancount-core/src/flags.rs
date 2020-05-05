use std::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq)]
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

impl<'a, I: Into<Cow<'a, str>>> From<I> for Flag<'a> {
    fn from(s: I) -> Self {
        let s = s.into();
        if s == "*" || s == "txn" {
            Flag::Okay
        } else if s == "!" {
            Flag::Warning
        } else {
            Flag::Other(s)
        }
    }
}
