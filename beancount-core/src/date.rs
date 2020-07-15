use std::borrow::Cow;
use std::{fmt, fmt::Display};

#[cfg(feature = "chrono")]
use chrono::NaiveDate;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Date<'a>(Cow<'a, str>);

impl<'a> From<Cow<'a, str>> for Date<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        Date(s)
    }
}

impl<'a> From<&'a str> for Date<'a> {
    fn from(s: &'a str) -> Self {
        Date(Cow::from(s))
    }
}

impl From<String> for Date<'_> {
    fn from(s: String) -> Self {
        Date(Cow::from(s))
    }
}

impl<'a> From<Date<'a>> for Cow<'a, str> {
    fn from(d: Date<'a>) -> Self {
        d.0
    }
}

impl Display for Date<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "chrono")]
impl From<NaiveDate> for Date<'_> {
    fn from(d: NaiveDate) -> Self {
        Cow::from(d.format("%Y-%m-%d").to_string()).into()
    }
}

#[cfg(feature = "chrono")]
#[test]
fn test_date_from_chrono() {
    assert_eq!(
        Date::from(chrono::NaiveDate::from_ymd(2020, 05, 05)),
        Cow::from("2020-05-05").into()
    );
}
