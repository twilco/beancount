use std::borrow::Cow;
use std::{fmt, fmt::Display};

#[cfg(feature = "chrono")]
use chrono::NaiveDate;

/// Represents a beancount date. It can be created using the `from_*_unchecked` methods.
/// Alternatively, with the `chrono` feature enabled, it can be converted from a `NaiveDate`.
/// 
/// # Example
/// ```rust
/// use beancount_core::Date;
/// 
/// // Create a Date from a String
/// let past: Date<'static> = Date::from_str_unchecked("2020-01-01");
/// let later: Date<'static> = Date::from_str_unchecked("43020-01-01");
/// assert!(later > past);
/// 
/// // Create a Date from a chrono type.
/// #[cfg(feature = "chrono")]
/// let today: Date<'static> = chrono::Local::today().naive_local().into();
/// ```
#[derive(Eq, PartialEq, Debug, Clone, Ord, PartialOrd, Hash)]
pub struct Date<'a>(Cow<'a, str>);

impl Date<'_> {
    pub fn from_str_unchecked(s: &str) -> Date<'_> {
        Date(s.into())
    }

    pub fn from_string_unchecked(s: String) -> Date<'static> {
        Date(s.into())
    }

    pub fn from_cow_unchecked(s: Cow<'_, str>) -> Date<'_> {
        Date(s)
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
        Date::from_string_unchecked(d.format("%Y-%m-%d").to_string())
    }
}

#[cfg(feature = "chrono")]
#[test]
fn test_date_from_chrono() {
    assert_eq!(
        Date::from(chrono::NaiveDate::from_ymd(2020, 05, 05)),
        Date::from_str_unchecked("2020-05-05")
    );
}
