//! Strict validation against a version's schema.
//!
//! A schema is a hand-written table of the lists, records and fields of one
//! BeerXML version (see [`v1_0`]); [`validate`] walks a document against it
//! and reports every [`Violation`].

use std::fmt;

use roxmltree::Node;

use crate::{ReadError, Version, encoding, reader};

mod v1_0;

/// A way a document breaks its version's schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Path to the offending element, e.g. `RECIPES/RECIPE[2]/HOPS/HOP[1]/ALPHA`.
    /// Records carry a 1-based index among the records of their list; fields
    /// carry none. For a missing element it is the path the element would
    /// have.
    pub path: String,
    /// The rule that is broken.
    pub rule: Rule,
    /// The element's full original text, or `None` when the element is
    /// missing.
    pub value: Option<String>,
}

/// A schema rule a [`Violation`] breaks.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    /// A required element is missing: a field of a record (e.g. a `HOP`
    /// without `ALPHA`), or the first record of a list that must hold at
    /// least one (e.g. an empty `HOPS`).
    Required,
    /// The value is not a decimal number (`xs:decimal`): an optional sign,
    /// digits with an optional fraction, and surrounding whitespace. Units
    /// (`5.5%`), exponents (`1e3`) and empty elements are violations.
    Decimal,
    /// The value is a decimal number below zero in a field the schema
    /// restricts to zero or more (amounts, percentages, times).
    NonNegative,
    /// The value is not a whole number in the 32-bit range (`xs:int`).
    Integer,
    /// The value is not exactly `TRUE` or `FALSE`.
    Boolean,
    /// The value is not exactly one of the spec's values, letter case and
    /// whitespace included.
    Enumeration {
        /// The values the spec allows, in its order.
        allowed: &'static [&'static str],
    },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.path)?;
        match &self.value {
            Some(value) => write!(f, "{value:?} {}", self.rule),
            None => write!(f, "{}", self.rule),
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Required => f.write_str("required element is missing"),
            Self::Decimal => f.write_str("is not a decimal number"),
            Self::NonNegative => f.write_str("is negative"),
            Self::Integer => f.write_str("is not an integer"),
            Self::Boolean => f.write_str("is not TRUE or FALSE"),
            Self::Enumeration { allowed } => write!(f, "is not one of {allowed:?}"),
        }
    }
}

/// Validates a BeerXML file strictly against the schema of `version`.
///
/// Returns every [`Violation`] in the document; an empty list means the file
/// is valid. Use it to check the output of an exporter; use [`read`](crate::read)
/// to import files from other programs.
///
/// The BeerXML 1.0 rules are written by hand from `beerxml-1.0.xsd` in this
/// repository:
///
/// - **Required fields** of every record type, and the first record of each
///   list the schema requires to be non-empty, must be present.
/// - **Numbers** must be plain decimals (or integers, for `VERSION`,
///   `TIMES_CULTURED`, `MAX_REUSE` and `FERMENTATION_STAGES`), with nothing
///   else but surrounding whitespace. Fields the XSD types as `decimalType`
///   or `timeType` (amounts, percentages, times, …) must not be negative;
///   temperatures and gravities may be.
/// - **Booleans** must be exactly `TRUE` or `FALSE`.
/// - **Enumerated fields** must spell one of the spec's values exactly, so
///   `Dry hop` or `Boil ` are violations, though [`read`](crate::read)
///   accepts them.
///
/// Not checked, as the 1.0 spec lets writers extend files: unknown elements
/// (including `DISPLAY_*` fields, which are plain text) and the order of
/// elements. A field that appears twice has each occurrence checked.
/// Elements are matched by local name, ignoring namespaces, as in
/// [`read`](crate::read).
///
/// # Errors
///
/// The same fatal problems as [`read`](crate::read) return a [`ReadError`]:
/// the input is not XML, its encoding is not UTF-8 or ISO-8859-1, or its root
/// element is not a BeerXML list. [`ReadError::UnsupportedVersion`] is
/// returned for [`Version::V1_1`], whose rules are not written yet.
pub fn validate(bytes: &[u8], version: Version) -> Result<Vec<Violation>, ReadError> {
    let schema = match version {
        Version::V1_0 => v1_0::LISTS,
        Version::V1_1 => return Err(ReadError::UnsupportedVersion { version }),
    };
    let text = encoding::decode(bytes)?;
    let doc = reader::parse(&text)?;
    let root = doc.root_element();
    let name = root.tag_name().name();
    let list = schema
        .iter()
        .find(|l| l.name == name)
        .ok_or_else(|| ReadError::NoBeerXmlRoot {
            found: name.to_owned(),
        })?;
    let mut violations = Vec::new();
    check_list(root, name, list, &mut violations);
    Ok(violations)
}

/// A list element (e.g. `HOPS`) and the records it holds.
struct List {
    name: &'static str,
    record: &'static str,
    fields: &'static [Field],
    /// Whether the list must hold at least one record.
    non_empty: bool,
}

/// A field of a record and its type.
struct Field {
    name: &'static str,
    required: bool,
    kind: Kind,
}

/// The type of a field's content.
enum Kind {
    Text,
    Decimal,
    NonNegativeDecimal,
    Integer,
    Boolean,
    Enumeration(&'static [&'static str]),
    /// A single nested record, such as a recipe's `STYLE`.
    Record(&'static [Field]),
    /// A nested list, such as a recipe's `HOPS`.
    List(&'static List),
}

fn check_list(node: Node<'_, '_>, path: &str, list: &List, out: &mut Vec<Violation>) {
    let mut count = 0;
    for child in node.children().filter(Node::is_element) {
        if child.tag_name().name() == list.record {
            count += 1;
            let record_path = format!("{path}/{}[{count}]", list.record);
            check_record(child, &record_path, list.fields, out);
        }
    }
    if list.non_empty && count == 0 {
        out.push(missing(format!("{path}/{}", list.record)));
    }
}

fn check_record(node: Node<'_, '_>, path: &str, fields: &[Field], out: &mut Vec<Violation>) {
    for child in node.children().filter(Node::is_element) {
        let name = child.tag_name().name();
        if let Some(field) = fields.iter().find(|f| f.name == name) {
            check_field(child, &format!("{path}/{name}"), &field.kind, out);
        }
    }
    for field in fields.iter().filter(|f| f.required) {
        let present = node
            .children()
            .any(|c| c.is_element() && c.tag_name().name() == field.name);
        if !present {
            out.push(missing(format!("{path}/{}", field.name)));
        }
    }
}

fn check_field(node: Node<'_, '_>, path: &str, kind: &Kind, out: &mut Vec<Violation>) {
    let text = node.text().unwrap_or("");
    let rule = match kind {
        Kind::Text => None,
        Kind::Decimal => (!is_decimal(text)).then_some(Rule::Decimal),
        Kind::NonNegativeDecimal => decimal_at_least_zero(text),
        Kind::Integer => (!is_int(text)).then_some(Rule::Integer),
        Kind::Boolean => (!matches!(text, "TRUE" | "FALSE")).then_some(Rule::Boolean),
        Kind::Enumeration(allowed) => {
            (!allowed.contains(&text)).then_some(Rule::Enumeration { allowed })
        }
        Kind::Record(fields) => return check_record(node, path, fields, out),
        Kind::List(list) => return check_list(node, path, list, out),
    };
    if let Some(rule) = rule {
        out.push(Violation {
            path: path.to_owned(),
            rule,
            value: Some(text.to_owned()),
        });
    }
}

fn missing(path: String) -> Violation {
    Violation {
        path,
        rule: Rule::Required,
        value: None,
    }
}

/// Strips the whitespace XML Schema collapses around numbers.
fn collapse(text: &str) -> &str {
    text.trim_matches([' ', '\t', '\n', '\r'])
}

/// Whether `text` is in the lexical space of `xs:decimal`:
/// `[+-]?(digits(.digits?)?|.digits)`, surrounded by whitespace.
fn is_decimal(text: &str) -> bool {
    let s = collapse(text);
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    let all_digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
    !(int.is_empty() && frac.is_empty()) && all_digits(int) && all_digits(frac)
}

fn decimal_at_least_zero(text: &str) -> Option<Rule> {
    if !is_decimal(text) {
        return Some(Rule::Decimal);
    }
    // Any `xs:decimal` parses as an `f64`; `-0` is zero, not negative.
    let value: f64 = collapse(text).parse().unwrap_or_default();
    (value < 0.0).then_some(Rule::NonNegative)
}

/// Whether `text` is an `xs:int`: a whole number in the 32-bit range.
fn is_int(text: &str) -> bool {
    collapse(text).parse::<i32>().is_ok()
}
