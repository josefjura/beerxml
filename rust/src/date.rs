//! Dates as BeerXML records hold them.

use std::fmt;

/// A date field: the original text, plus the date itself when the text is
/// an ISO `YYYY-MM-DD` date.
///
/// BeerXML 1.0 leaves date formats free, so files hold anything from
/// `2003-07-06` to `7/6/2003` or `3 Dec 04`. The text is always kept; only an
/// unambiguous ISO date is parsed.
///
/// ```
/// use beerxml::{IsoDate, RecordDate};
/// let iso = RecordDate::parse("2003-07-06");
/// assert_eq!(iso.iso, Some(IsoDate { year: 2003, month: 7, day: 6 }));
/// assert_eq!(RecordDate::parse("7/6/2003").iso, None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordDate {
    /// The original text, with surrounding whitespace trimmed.
    pub text: String,
    /// The date, when the text is a valid ISO `YYYY-MM-DD` date.
    pub iso: Option<IsoDate>,
}

impl RecordDate {
    /// Reads a date field: trims `text` and parses it when it is an ISO date.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let text = text.trim();
        Self {
            text: text.to_owned(),
            iso: IsoDate::parse(text),
        }
    }
}

/// A calendar date in the proleptic Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IsoDate {
    /// The year, e.g. `2003`.
    pub year: i32,
    /// The month, `1` to `12`.
    pub month: u8,
    /// The day of the month, `1` to `31`.
    pub day: u8,
}

impl IsoDate {
    /// Parses exactly `YYYY-MM-DD`, rejecting dates that don't exist such as
    /// `2003-02-30`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let b = text.as_bytes();
        let all_digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        if !(all_digits(0..4) && all_digits(5..7) && all_digits(8..10)) {
            return None;
        }
        let year = text[0..4].parse().ok()?;
        let month = text[5..7].parse().ok()?;
        let day = text[8..10].parse().ok()?;
        (1..=12)
            .contains(&month)
            .then(|| days_in_month(year, month))
            .filter(|&days| (1..=days).contains(&day))
            .map(|_| Self { year, month, day })
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl fmt::Display for IsoDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::{IsoDate, RecordDate};

    #[allow(clippy::unnecessary_wraps)] // compared against `Option` results
    fn date(year: i32, month: u8, day: u8) -> Option<IsoDate> {
        Some(IsoDate { year, month, day })
    }

    #[test]
    fn iso_dates_are_parsed() {
        assert_eq!(IsoDate::parse("2003-07-06"), date(2003, 7, 6));
        assert_eq!(IsoDate::parse("2000-02-29"), date(2000, 2, 29));
        assert_eq!(IsoDate::parse("2024-12-31"), date(2024, 12, 31));
    }

    #[test]
    fn other_text_is_not_a_date() {
        for text in [
            "7/6/2003",
            "3 Dec 04",
            "2003-7-6",
            "2003-07-06T10:00",
            "+003-07-06",
            "2003-13-01",
            "2003-00-10",
            "2003-02-29",
            "1900-02-29",
            "2003-04-31",
            "2003-01-00",
            "",
        ] {
            assert_eq!(IsoDate::parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn record_date_keeps_trimmed_text() {
        let d = RecordDate::parse(" 3 Dec 04 \n");
        assert_eq!(d.text, "3 Dec 04");
        assert_eq!(d.iso, None);
        assert_eq!(RecordDate::parse(" 2003-07-06 ").iso, date(2003, 7, 6));
    }

    #[test]
    fn display_is_iso() {
        assert_eq!(
            IsoDate {
                year: 812,
                month: 3,
                day: 4
            }
            .to_string(),
            "0812-03-04"
        );
    }
}
