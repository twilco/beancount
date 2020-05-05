use std::borrow::Cow;

#[cfg(feature = "chrono")]
use chrono::NaiveDate;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Date<'a> {
    s: Cow<'a, str>
}

impl<'a, I: Into<Cow<'a, str>>> From<I> for Date<'a> {
    fn from(d: I) -> Self {
        Date { s: d.into() }
    }
}

#[cfg(feature = "chrono")]
impl From<NaiveDate> for Date<'static> {
    fn from(d: NaiveDate) -> Self {
        Date { s: d.format("%Y-%m-%d").to_string() }
    }
}
