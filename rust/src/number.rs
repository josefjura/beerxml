/// Result of reading the leading decimal number of a field.
#[derive(Debug, PartialEq)]
pub(crate) enum Leading {
    /// The text was empty or whitespace only.
    Empty,
    /// A number was read; `trailing` is true when non-whitespace text followed.
    Number { value: f64, trailing: bool },
    /// The text does not start with a number.
    Unreadable,
}

/// Reads the leading decimal number of `text`, ignoring surrounding
/// whitespace: an optional sign, digits with an optional fraction (or a bare
/// fraction like `.5`), and an optional exponent.
pub(crate) fn leading(text: &str) -> Leading {
    let s = text.trim();
    if s.is_empty() {
        return Leading::Empty;
    }
    let b = s.as_bytes();
    let digits = |mut i: usize| {
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        i
    };
    let mut i = 0;
    if matches!(b.first(), Some(b'+' | b'-')) {
        i += 1;
    }
    let int_end = digits(i);
    let mut end = int_end;
    let mut has_digits = int_end > i;
    if b.get(end) == Some(&b'.') {
        let frac_end = digits(end + 1);
        has_digits |= frac_end > end + 1;
        end = frac_end;
    }
    if !has_digits {
        return Leading::Unreadable;
    }
    if matches!(b.get(end), Some(b'e' | b'E')) {
        let mut j = end + 1;
        if matches!(b.get(j), Some(b'+' | b'-')) {
            j += 1;
        }
        let exp_end = digits(j);
        if exp_end > j {
            end = exp_end;
        }
    }
    match s[..end].parse::<f64>() {
        Ok(value) => Leading::Number {
            value,
            trailing: end < s.len(),
        },
        Err(_) => Leading::Unreadable,
    }
}

#[cfg(test)]
mod tests {
    use super::{Leading, leading};

    fn num(value: f64, trailing: bool) -> Leading {
        Leading::Number { value, trailing }
    }

    #[test]
    fn plain_and_padded_numbers() {
        assert_eq!(leading("30.000000"), num(30.0, false));
        assert_eq!(leading("  1.043\n"), num(1.043, false));
        assert_eq!(leading("-2"), num(-2.0, false));
        assert_eq!(leading("+.5"), num(0.5, false));
        assert_eq!(leading("5."), num(5.0, false));
        assert_eq!(leading("1.5e2"), num(150.0, false));
    }

    #[test]
    fn trailing_text() {
        assert_eq!(leading("34.2 IBU"), num(34.2, true));
        assert_eq!(leading("75%"), num(75.0, true));
        assert_eq!(leading("1,043"), num(1.0, true));
        // An `e` not followed by digits is trailing text, not an exponent.
        assert_eq!(leading("2 each"), num(2.0, true));
        assert_eq!(leading("2e"), num(2.0, true));
    }

    #[test]
    fn empty_and_unreadable() {
        assert_eq!(leading(""), Leading::Empty);
        assert_eq!(leading("  \t"), Leading::Empty);
        assert_eq!(leading("abc"), Leading::Unreadable);
        assert_eq!(leading("."), Leading::Unreadable);
        assert_eq!(leading("-"), Leading::Unreadable);
        assert_eq!(leading("NaN"), Leading::Unreadable);
        assert_eq!(leading("inf"), Leading::Unreadable);
    }
}
