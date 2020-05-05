use std::borrow::Cow;

#[cfg(feature = "chrono")]
use chrono::NaiveDate;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Date<'a> {
    s: Cow<'a, str>
}

impl<'a> From<Cow<'a, str>> for Date<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        Date { s }
    }
}

#[cfg(feature = "chrono")]
impl From<NaiveDate> for Date<'static> {
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
