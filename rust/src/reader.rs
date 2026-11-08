use roxmltree::Node;

use crate::enums::Lenient;
use crate::number::{Leading, leading};
use crate::units::{Kilograms, Litres, Minutes, Percent, SpecificGravity};
use crate::{Document, Hop, ReadError, Recipe, Version, Warning, WarningKind};

/// The result of reading a file leniently.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Read {
    /// The records the file contains.
    pub document: Document,
    /// The BeerXML version the file was detected as.
    pub version: Version,
    /// Every repair made while reading, in document order.
    pub warnings: Vec<Warning>,
}

/// Reads a BeerXML 1.0 or 1.1 file leniently.
///
/// Any real-world file that is well-formed XML with a BeerXML root element
/// reads into a [`Document`]; every repair is reported in
/// [`Read::warnings`]. See the [crate-level docs](crate) for the rules.
///
/// # Errors
///
/// Returns a [`ReadError`] only for fatal problems: the input is not XML, or
/// its root element is not a BeerXML top-level list.
pub fn read(bytes: &[u8]) -> Result<Read, ReadError> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|e| ReadError::NotXml {
        message: format!("not valid UTF-8: {e}"),
    })?;
    let doc = roxmltree::Document::parse(text).map_err(|e| ReadError::NotXml {
        message: e.to_string(),
    })?;

    let root = doc.root_element();
    let root_name = root.tag_name().name();
    let Some(record_name) = record_name_of_list(root_name) else {
        return Err(ReadError::NoBeerXmlRoot {
            found: root_name.to_owned(),
        });
    };

    let mut reader = Reader {
        version: Version::detect(&doc),
        warnings: Vec::new(),
    };
    let mut document = Document::default();
    match root_name {
        "RECIPES" => document.recipes = reader.list(root, root_name, record_name, Reader::recipe),
        "HOPS" => document.hops = reader.list(root, root_name, record_name, Reader::hop),
        // Not modelled yet: still report unknown elements around the records.
        _ => {
            reader.list(root, root_name, record_name, |_, _, _| ());
        }
    }
    Ok(Read {
        document,
        version: reader.version,
        warnings: reader.warnings,
    })
}

/// The record element each BeerXML top-level list holds.
fn record_name_of_list(list: &str) -> Option<&'static str> {
    Some(match list {
        "RECIPES" => "RECIPE",
        "HOPS" => "HOP",
        "FERMENTABLES" => "FERMENTABLE",
        "YEASTS" => "YEAST",
        "MISCS" => "MISC",
        "WATERS" => "WATER",
        "STYLES" => "STYLE",
        "EQUIPMENTS" => "EQUIPMENT",
        "MASHS" => "MASH",
        "MASH_STEPS" => "MASH_STEP",
        _ => return None,
    })
}

/// Every element a `RECIPE` may contain in the BeerXML 1.1 schema, used to
/// tell fields that aren't modelled yet from unknown elements.
const RECIPE_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "TYPE",
    "BREWER",
    "ASST_BREWER",
    "BATCH_SIZE",
    "BOIL_SIZE",
    "BOIL_TIME",
    "EFFICIENCY",
    "STYLE",
    "EQUIPMENT",
    "HOPS",
    "FERMENTABLES",
    "MISCS",
    "YEASTS",
    "WATERS",
    "MASH",
    "NOTES",
    "TASTE_NOTES",
    "TASTE_RATING",
    "RATING",
    "OG",
    "FG",
    "FERMENTATION_STAGES",
    "PRIMARY_AGE",
    "PRIMARY_TEMP",
    "SECONDARY_AGE",
    "SECONDARY_TEMP",
    "TERTIARY_AGE",
    "TERTIARY_TEMP",
    "AGE",
    "AGE_TEMP",
    "DATE",
    "CARBONATION",
    "FORCED_CARBONATION",
    "PRIMING_SUGAR_NAME",
    "CARBONATION_TEMP",
    "PRIMING_SUGAR_EQUIV",
    "KEG_PRIMING_FACTOR",
    "IBU",
    "IBUS",
    "IBU_METHOD",
    "EST_ABV",
    "ABV",
    "ACTUAL_EFFICIENCY",
    "CALORIES",
    "CARBONATION_USED",
];

/// Every element a `HOP` may contain in the BeerXML 1.1 schema.
const HOP_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "ALPHA",
    "AMOUNT",
    "USE",
    "TIME",
    "NOTES",
    "TYPE",
    "FORM",
    "BETA",
    "HSI",
    "ORIGIN",
    "SUBSTITUTES",
    "HUMULENE",
    "CARYOPHYLLENE",
    "COHUMULONE",
    "MYRCENE",
];

/// A field of a record being read: its element name, text and path.
struct Field<'a> {
    name: &'a str,
    text: &'a str,
    node: Node<'a, 'a>,
    path: String,
}

struct Reader {
    version: Version,
    warnings: Vec<Warning>,
}

impl Reader {
    fn warn(&mut self, path: impl Into<String>, kind: WarningKind) {
        self.warnings.push(Warning {
            path: path.into(),
            kind,
        });
    }

    /// Reports an unknown element; only 1.1 documents report them, since the
    /// 1.0 spec requires readers to ignore unknown elements silently.
    fn unknown(&mut self, path: String) {
        if self.version == Version::V1_1 {
            self.warn(path, WarningKind::UnknownElement);
        }
    }

    /// Reads the `record_name` children of a list element with `read`, in
    /// document order. Each record's path carries its 1-based index. Other
    /// children are reported as unknown.
    fn list<T>(
        &mut self,
        list: Node<'_, '_>,
        list_path: &str,
        record_name: &str,
        mut read: impl FnMut(&mut Self, Node<'_, '_>, &str) -> T,
    ) -> Vec<T> {
        let mut records = Vec::new();
        for child in list.children().filter(Node::is_element) {
            let name = child.tag_name().name();
            if name == record_name {
                let path = format!("{list_path}/{name}[{}]", records.len() + 1);
                records.push(read(self, child, &path));
            } else if !name.starts_with("DISPLAY_") {
                self.unknown(format!("{list_path}/{name}"));
            }
        }
        records
    }

    /// Walks the fields of a record and returns its `NAME`.
    ///
    /// `field` handles a modelled field and returns `true`. Unhandled fields
    /// are skipped when they are `DISPLAY_*` or in `known` (part of the
    /// schema but not modelled yet), and reported as unknown otherwise.
    fn record(
        &mut self,
        node: Node<'_, '_>,
        path: &str,
        known: &[&str],
        mut field: impl FnMut(&mut Self, &Field<'_>) -> bool,
    ) -> String {
        let mut name = String::new();
        for child in node.children().filter(Node::is_element) {
            let f = Field {
                name: child.tag_name().name(),
                text: child.text().unwrap_or(""),
                node: child,
                path: format!("{path}/{}", child.tag_name().name()),
            };
            if f.name == "NAME" {
                f.text.trim().clone_into(&mut name);
            } else if !field(self, &f)
                && !f.name.starts_with("DISPLAY_")
                && !known.contains(&f.name)
            {
                self.unknown(f.path);
            }
        }
        if name.is_empty() {
            self.warn(format!("{path}/NAME"), WarningKind::MissingName);
        }
        name
    }

    fn recipe(&mut self, node: Node<'_, '_>, path: &str) -> Recipe {
        let mut r = Recipe::default();
        r.name = self.record(node, path, RECIPE_ELEMENTS, |s, f| {
            match f.name {
                "BATCH_SIZE" => r.batch_size = s.number(f).map(Litres::new),
                "BOIL_SIZE" => r.boil_size = s.number(f).map(Litres::new),
                "BOIL_TIME" => r.boil_time = s.number(f).map(Minutes::new),
                "EFFICIENCY" => r.efficiency = s.number(f).map(Percent::new),
                "OG" => r.og = s.number(f).map(SpecificGravity::new),
                "FG" => r.fg = s.number(f).map(SpecificGravity::new),
                "HOPS" => r.hops = s.list(f.node, &f.path, "HOP", Self::hop),
                _ => return false,
            }
            true
        });
        r
    }

    fn hop(&mut self, node: Node<'_, '_>, path: &str) -> Hop {
        let mut h = Hop::default();
        h.name = self.record(node, path, HOP_ELEMENTS, |s, f| {
            match f.name {
                "ALPHA" => h.alpha = s.number(f).map(Percent::new),
                "AMOUNT" => h.amount = s.number(f).map(Kilograms::new),
                "USE" => h.usage = s.enumerated(f),
                "TIME" => h.time = s.number(f).map(Minutes::new),
                "NOTES" => h.notes = text(f),
                "TYPE" => h.kind = s.enumerated(f),
                "FORM" => h.form = s.enumerated(f),
                "BETA" => h.beta = s.number(f).map(Percent::new),
                "HSI" => h.hsi = s.number(f).map(Percent::new),
                "ORIGIN" => h.origin = text(f),
                "SUBSTITUTES" => h.substitutes = text(f),
                "HUMULENE" => h.humulene = s.number(f).map(Percent::new),
                "CARYOPHYLLENE" => h.caryophyllene = s.number(f).map(Percent::new),
                "COHUMULONE" => h.cohumulone = s.number(f).map(Percent::new),
                "MYRCENE" => h.myrcene = s.number(f).map(Percent::new),
                _ => return false,
            }
            true
        });
        h
    }

    /// Reads a numeric field by the leading-number rule.
    fn number(&mut self, f: &Field<'_>) -> Option<f64> {
        match leading(f.text) {
            Leading::Empty => None,
            Leading::Number { value, trailing } => {
                if trailing {
                    let text = f.text.to_owned();
                    self.warn(f.path.clone(), WarningKind::TrailingText { text });
                }
                Some(value)
            }
            Leading::Unreadable => {
                let text = f.text.to_owned();
                self.warn(f.path.clone(), WarningKind::UnreadableNumber { text });
                None
            }
        }
    }

    /// Reads an enumerated field; an empty field is absent.
    fn enumerated<E: Lenient>(&mut self, f: &Field<'_>) -> Option<E> {
        if f.text.trim().is_empty() {
            return None;
        }
        let value = E::parse(f.text);
        if value.is_other() {
            let text = f.text.to_owned();
            self.warn(f.path.clone(), WarningKind::UnknownValue { text });
        }
        Some(value)
    }
}

/// Reads a text field, trimmed; an empty field is absent.
fn text(f: &Field<'_>) -> Option<String> {
    let text = f.text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}
