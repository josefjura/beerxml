//! Turning the raw bytes of a file into text, per its character encoding.
//!
//! Only UTF-8 and ISO-8859-1 are supported: BeerXML 1.0 writers (notably old
//! BeerSmith versions) use one or the other. ISO-8859-1 is transcoded by
//! hand, since each of its bytes is the Unicode code point of the same value.

use std::borrow::Cow;

use crate::ReadError;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// Declared encoding names read as UTF-8, lower-cased. US-ASCII is a subset of
/// UTF-8, so an ASCII file reads unchanged.
const UTF8_NAMES: &[&str] = &["utf-8", "utf8", "us-ascii", "ascii"];

/// Declared encoding names read as ISO-8859-1, lower-cased: the IANA name and
/// aliases, plus common misspellings of them.
const LATIN1_NAMES: &[&str] = &[
    "iso-8859-1",
    "iso_8859-1",
    "iso8859-1",
    "iso_8859-1:1987",
    "iso-ir-100",
    "latin1",
    "latin-1",
    "l1",
    "ibm819",
    "cp819",
    "csisolatin1",
];

/// Decodes `bytes` into text.
///
/// - A UTF-8 byte order mark means UTF-8, whatever the declaration says (XML
///   1.0, Appendix F.1); the mark itself is dropped.
/// - A UTF-16 byte order mark is refused.
/// - Otherwise the `encoding` of the XML declaration decides, matched
///   case-insensitively. No declaration, or no `encoding` in it, means UTF-8
///   (XML 1.0, section 4.3.3).
///
/// # Errors
///
/// [`ReadError::UnsupportedEncoding`] for an encoding other than UTF-8 or
/// ISO-8859-1, and [`ReadError::NotXml`] for bytes that aren't valid UTF-8 in
/// a UTF-8 file.
pub(crate) fn decode(bytes: &[u8]) -> Result<Cow<'_, str>, ReadError> {
    if let Some(rest) = bytes.strip_prefix(UTF8_BOM) {
        return utf8(rest);
    }
    match bytes {
        [0xFF, 0xFE, ..] => return Err(unsupported("UTF-16LE")),
        [0xFE, 0xFF, ..] => return Err(unsupported("UTF-16BE")),
        _ => {}
    }
    let Some(declared) = declared_encoding(bytes) else {
        return utf8(bytes);
    };
    let name = declared.to_ascii_lowercase();
    if UTF8_NAMES.contains(&name.as_str()) {
        utf8(bytes)
    } else if LATIN1_NAMES.contains(&name.as_str()) {
        Ok(Cow::Owned(bytes.iter().copied().map(char::from).collect()))
    } else {
        Err(unsupported(&declared))
    }
}

fn utf8(bytes: &[u8]) -> Result<Cow<'_, str>, ReadError> {
    std::str::from_utf8(bytes)
        .map(Cow::Borrowed)
        .map_err(|e| ReadError::NotXml {
            message: format!("not valid UTF-8: {e}"),
        })
}

fn unsupported(encoding: &str) -> ReadError {
    ReadError::UnsupportedEncoding {
        encoding: encoding.to_owned(),
    }
}

/// The `encoding` pseudo-attribute of the XML declaration at the very start
/// of `bytes` (XML 1.0, section 2.8 `EncodingDecl`), if there is one.
///
/// A malformed declaration yields `None`, so the file is read as UTF-8 and
/// the XML parser reports the problem.
fn declared_encoding(bytes: &[u8]) -> Option<String> {
    let rest = bytes.strip_prefix(b"<?xml")?;
    if !rest.first().is_some_and(|&b| is_space(b)) {
        return None;
    }
    let end = rest.windows(2).position(|w| w == b"?>")?;
    let decl = &rest[..end];

    let mut i = 0;
    while i < decl.len() {
        // Pseudo-attribute name.
        while decl.get(i).is_some_and(|&b| is_space(b)) {
            i += 1;
        }
        let name_start = i;
        while decl.get(i).is_some_and(|&b| b != b'=' && !is_space(b)) {
            i += 1;
        }
        let name = &decl[name_start..i];
        // `=`, with optional whitespace around it.
        while decl.get(i).is_some_and(|&b| is_space(b)) {
            i += 1;
        }
        if decl.get(i) != Some(&b'=') {
            return None;
        }
        i += 1;
        while decl.get(i).is_some_and(|&b| is_space(b)) {
            i += 1;
        }
        // Quoted value.
        let quote = *decl.get(i).filter(|&&b| b == b'"' || b == b'\'')?;
        let value_start = i + 1;
        let value_len = decl[value_start..].iter().position(|&b| b == quote)?;
        let value = &decl[value_start..value_start + value_len];
        i = value_start + value_len + 1;

        if name == b"encoding" {
            return std::str::from_utf8(value).ok().map(str::to_owned);
        }
    }
    None
}

/// XML whitespace (XML 1.0, section 2.3 `S`).
fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n')
}

#[cfg(test)]
mod tests {
    use super::declared_encoding;

    #[test]
    fn finds_the_encoding_after_other_pseudo_attributes() {
        assert_eq!(
            declared_encoding(br#"<?xml version="1.0" encoding='Latin1' standalone="no"?><A/>"#),
            Some("Latin1".to_owned())
        );
    }

    #[test]
    fn no_declaration_or_no_encoding_is_none() {
        assert_eq!(declared_encoding(b"<HOPS/>"), None);
        assert_eq!(declared_encoding(br#"<?xml version="1.0"?><A/>"#), None);
        assert_eq!(
            declared_encoding(b" <?xml version='1.0' encoding='latin1'?>"),
            None
        );
        assert_eq!(declared_encoding(b"<?xml-stylesheet href='a'?>"), None);
    }

    #[test]
    fn malformed_declaration_is_none() {
        assert_eq!(
            declared_encoding(b"<?xml version encoding='latin1'?>"),
            None
        );
        assert_eq!(
            declared_encoding(b"<?xml version='1.0' encoding='latin1"),
            None
        );
        assert_eq!(
            declared_encoding(b"<?xml version='1.0' encoding=latin1?>"),
            None
        );
    }

    #[test]
    fn encoding_inside_another_value_is_not_the_encoding() {
        assert_eq!(
            declared_encoding(br#"<?xml version="encoding='latin1'"?><A/>"#),
            None
        );
    }
}
