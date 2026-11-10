//! Strict validation against a version's schema.
//!
//! A schema is a hand-written table of the lists, records and fields of one
//! BeerXML version (see [`v1_0`] and [`v1_1`]); [`validate`] walks a document
//! against it and reports every [`Violation`].

use std::fmt;

use roxmltree::Node;

use crate::encoding::{self, UTF8_BOM};
use crate::{ReadError, Version, date::IsoDate, reader};

mod shared;
mod v1_0;
mod v1_1;

/// The namespace of the 1.1 XSD (its `targetNamespace`).
const NAMESPACE_1_1: &str = "http://beerxml.com/v1.1";

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
    /// missing. For the document-wide [`Rule::Namespace`] and [`Rule::Utf8`]
    /// it is the namespace or declared encoding found instead, with `path`
    /// naming the root element.
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
    /// The value is not exactly `TRUE` or `FALSE` (BeerXML 1.0).
    Boolean,
    /// The value is not exactly one of the spec's values, letter case and
    /// whitespace included.
    Enumeration {
        /// The values the spec allows, in its order.
        allowed: &'static [&'static str],
    },
    /// The value is not an `xs:boolean`: `true`, `false`, `1` or `0`, with
    /// surrounding whitespace (BeerXML 1.1). `TRUE` is the 1.0 spelling.
    LowercaseBoolean,
    /// The value is not an ISO 8601 calendar date `YYYY-MM-DD`, with
    /// surrounding whitespace (BeerXML 1.1, RFC 01 section 3).
    Date,
    /// The element is a `DISPLAY_*` field, which BeerXML 1.1 removes (RFC 01
    /// section 2).
    DisplayField,
    /// A record's `VERSION` is not the decimal 1.1 that the 1.1 XSD fixes it
    /// to (RFC 01 section 5).
    Version,
    /// The root element is not in the 1.1 namespace `http://beerxml.com/v1.1`
    /// (RFC 01 section 5; the 1.1 XSD's `targetNamespace`). The value is the
    /// root's namespace, or `None` when it has none.
    Namespace,
    /// The XML declaration names an encoding other than UTF-8 (BeerXML 1.1,
    /// RFC 01 section 4). The value is the declared name.
    Utf8,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.path)?;
        if let (Rule::Namespace, None) = (&self.rule, &self.value) {
            return write!(f, "root element is not in the namespace {NAMESPACE_1_1}");
        }
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
            Self::LowercaseBoolean => f.write_str("is not true, false, 1 or 0"),
            Self::Date => f.write_str("is not an ISO date (YYYY-MM-DD)"),
            Self::DisplayField => f.write_str("is a display field, which 1.1 removes"),
            Self::Version => f.write_str("is not 1.1"),
            Self::Namespace => write!(f, "is not the namespace {NAMESPACE_1_1}"),
            Self::Utf8 => f.write_str("is not UTF-8"),
        }
    }
}

/// Validates a BeerXML file strictly against the schema of `version`.
///
/// Returns every [`Violation`] in the document; an empty list means the file
/// is valid. Use it to check the output of an exporter; use [`read`](crate::read)
/// to import files from other programs.
///
/// The rules are written by hand from the XSDs in this repository,
/// `beerxml-1.0.xsd` and `beerxml-1.1.xsd`. Both versions check:
///
/// - **Required fields** of every record type, and the first record of each
///   list the schema requires to be non-empty (1.0 only; 1.1 lists may be
///   empty), must be present.
/// - **Numbers** must be plain decimals (or integers, for `TIMES_CULTURED`,
///   `MAX_REUSE`, `FERMENTATION_STAGES` and the 1.0 `VERSION`), with nothing
///   else but surrounding whitespace, so a unit such as `34.2 IBU` is a
///   violation. Fields the XSD types as `decimalType` or `timeType`
///   (amounts, percentages, times, …) must not be negative; temperatures and
///   gravities may be.
/// - **Enumerated fields** must spell one of the spec's values exactly, so
///   `Dry hop` or `Boil ` are violations, though [`read`](crate::read)
///   accepts them.
///
/// In **1.0**, booleans must be exactly `TRUE` or `FALSE`, and the display
/// fields of the spec's Appendix A (`EST_OG`, `IBU`, `ABV`, …) are free text,
/// as the 1.0 XSD types them.
///
/// In **1.1**, the "Clean XML" rules of RFC 01
/// (`docs/spec/v1.1/RFC_01_goals.md`) apply on top:
///
/// - Every numeric field is a number, the former Appendix A display fields
///   included (section 1).
/// - No `DISPLAY_*` element may appear in a record or list (section 2), as
///   [`Rule::DisplayField`]. Other 1.0 display extensions not named
///   `DISPLAY_*` (`DISP_MIN_TEMP`, `INVENTORY`) are unknown elements.
/// - `DATE` and `CULTURE_DATE` are ISO `YYYY-MM-DD` dates (section 3), as
///   [`Rule::Date`]; like `xs:date`, there is no year 0000.
/// - Booleans are `xs:boolean`: `true`, `false`, `1` or `0`
///   ([`Rule::LowercaseBoolean`]).
/// - The file is UTF-8 (section 4): an XML declaration, if present, must
///   name `UTF-8`, as [`Rule::Utf8`]. A file without a declaration is read
///   as UTF-8, so bytes that aren't UTF-8 there are a
///   [`ReadError::NotXml`], not a violation.
/// - **The version marker** (section 5) is what the 1.1 XSD makes of it: the
///   root element is in the XSD's `targetNamespace` `http://beerxml.com/v1.1`
///   ([`Rule::Namespace`]), and every record's required `VERSION` is the
///   decimal 1.1 the XSD fixes it to (`fixed="1.1"`, so `1.10` is fine;
///   [`Rule::Version`]). The RFC's draft `version="1.1"` root attribute is
///   not required: the XSD declares no attributes. A file that passes is
///   always detected as 1.1 by [`read`](crate::read) (see [`Version`]).
///
/// Not checked in either version: unknown elements other than `DISPLAY_*`
/// (the 1.0 spec lets writers extend files, and the Python reference
/// validator ignores them in 1.1 too) and the order of elements. A field
/// that appears twice has each occurrence checked. Elements are matched by
/// local name, ignoring namespaces, as in [`read`](crate::read); only the
/// root's namespace is checked.
///
/// # Errors
///
/// The same fatal problems as [`read`](crate::read) return a [`ReadError`]:
/// the input is not XML, its encoding is not UTF-8 or ISO-8859-1, or its root
/// element is not a BeerXML list.
pub fn validate(bytes: &[u8], version: Version) -> Result<Vec<Violation>, ReadError> {
    let (schema, clean_xml) = match version {
        Version::V1_0 => (v1_0::LISTS, false),
        Version::V1_1 => (v1_1::LISTS, true),
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
    let mut walk = Walk {
        clean_xml,
        violations: Vec::new(),
    };
    if clean_xml {
        walk.check_document(bytes, root);
    }
    walk.check_list(root, name, list);
    Ok(walk.violations)
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
    /// `TRUE` or `FALSE` (1.0's `booleanType`).
    Boolean,
    /// `xs:boolean` (1.1).
    XsBoolean,
    /// `xs:date`, restricted to `YYYY-MM-DD` (1.1).
    Date,
    /// The decimal 1.1, the fixed `VERSION` of a 1.1 record.
    Version1_1,
    Enumeration(&'static [&'static str]),
    /// A single nested record, such as a recipe's `STYLE`.
    Record(&'static [Field]),
    /// A nested list, such as a recipe's `HOPS`.
    List(&'static List),
}

/// One validation pass over a document, collecting its violations.
struct Walk {
    /// Whether the 1.1 Clean XML rules apply.
    clean_xml: bool,
    violations: Vec<Violation>,
}

impl Walk {
    /// The document-wide 1.1 rules: UTF-8 and the namespace.
    fn check_document(&mut self, bytes: &[u8], root: Node<'_, '_>) {
        let path = root.tag_name().name();
        let declared = encoding::declared_encoding(bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes));
        if let Some(declared) = declared
            && !declared.eq_ignore_ascii_case("UTF-8")
        {
            self.push(path, Rule::Utf8, Some(declared));
        }
        let namespace = root.tag_name().namespace();
        if namespace != Some(NAMESPACE_1_1) {
            self.push(path, Rule::Namespace, namespace.map(str::to_owned));
        }
    }

    fn check_list(&mut self, node: Node<'_, '_>, path: &str, list: &List) {
        let mut count = 0;
        for child in node.children().filter(Node::is_element) {
            if child.tag_name().name() == list.record {
                count += 1;
                let record_path = format!("{path}/{}[{count}]", list.record);
                self.check_record(child, &record_path, list.fields);
            } else {
                self.check_unknown(child, path);
            }
        }
        if list.non_empty && count == 0 {
            self.push(&format!("{path}/{}", list.record), Rule::Required, None);
        }
    }

    fn check_record(&mut self, node: Node<'_, '_>, path: &str, fields: &[Field]) {
        for child in node.children().filter(Node::is_element) {
            let name = child.tag_name().name();
            let child_path = format!("{path}/{name}");
            if let Some(field) = fields.iter().find(|f| f.name == name) {
                self.check_field(child, &child_path, &field.kind);
            } else {
                self.check_unknown(child, path);
            }
        }
        for field in fields.iter().filter(|f| f.required) {
            let present = node
                .children()
                .any(|c| c.is_element() && c.tag_name().name() == field.name);
            if !present {
                self.push(&format!("{path}/{}", field.name), Rule::Required, None);
            }
        }
    }

    /// An element the schema doesn't know, inside `path`: allowed, except a
    /// `DISPLAY_*` field in 1.1.
    fn check_unknown(&mut self, node: Node<'_, '_>, path: &str) {
        let name = node.tag_name().name();
        if self.clean_xml && name.starts_with("DISPLAY_") {
            self.push(
                &format!("{path}/{name}"),
                Rule::DisplayField,
                Some(text_of(node)),
            );
        }
    }

    fn check_field(&mut self, node: Node<'_, '_>, path: &str, kind: &Kind) {
        let text = text_of(node);
        let rule = match kind {
            Kind::Text => None,
            Kind::Decimal => (!is_decimal(&text)).then_some(Rule::Decimal),
            Kind::NonNegativeDecimal => decimal_at_least_zero(&text),
            Kind::Integer => (!is_int(&text)).then_some(Rule::Integer),
            Kind::Boolean => (!matches!(text.as_str(), "TRUE" | "FALSE")).then_some(Rule::Boolean),
            Kind::XsBoolean => (!matches!(collapse(&text), "true" | "false" | "1" | "0"))
                .then_some(Rule::LowercaseBoolean),
            // `xs:date` has no year 0000.
            Kind::Date => IsoDate::parse(collapse(&text))
                .is_none_or(|d| d.year == 0)
                .then_some(Rule::Date),
            Kind::Version1_1 => (!is_one_point_one(&text)).then_some(Rule::Version),
            Kind::Enumeration(allowed) => {
                (!allowed.contains(&text.as_str())).then_some(Rule::Enumeration { allowed })
            }
            Kind::Record(fields) => return self.check_record(node, path, fields),
            Kind::List(list) => return self.check_list(node, path, list),
        };
        if let Some(rule) = rule {
            self.push(path, rule, Some(text));
        }
    }

    fn push(&mut self, path: &str, rule: Rule, value: Option<String>) {
        self.violations.push(Violation {
            path: path.to_owned(),
            rule,
            value,
        });
    }
}

/// The element's text, joining the text around any comments or processing
/// instructions inside it, as XML Schema does.
fn text_of(node: Node<'_, '_>) -> String {
    node.children()
        .filter(Node::is_text)
        .filter_map(|c| c.text())
        .collect()
}

/// Strips the whitespace XML Schema collapses around numbers, booleans and
/// dates.
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
    // Decided on the digits, not an `f64`, so a tiny negative value can't
    // round to zero; `-0` and `-0.0` are zero, not negative.
    let negative = collapse(text)
        .strip_prefix('-')
        .is_some_and(|digits| digits.bytes().any(|b| matches!(b, b'1'..=b'9')));
    negative.then_some(Rule::NonNegative)
}

/// Whether `text` is an `xs:decimal` equal to 1.1, compared by value as the
/// XSD's `fixed="1.1"` does: `1.10`, `+1.1` and `01.1` are 1.1 too.
fn is_one_point_one(text: &str) -> bool {
    if !is_decimal(text) {
        return false;
    }
    let s = collapse(text);
    let s = s.strip_prefix('+').unwrap_or(s);
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    int.trim_start_matches('0') == "1" && frac.trim_end_matches('0') == "1"
}

/// Whether `text` is an `xs:int`: a whole number in the 32-bit range.
fn is_int(text: &str) -> bool {
    collapse(text).parse::<i32>().is_ok()
}
