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
