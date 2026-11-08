use roxmltree::Node;

use crate::number::{Leading, leading};
use crate::{Document, ReadError, Recipe, Version, Warning, WarningKind};

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
    let mut index = 0;
    for child in root.children().filter(Node::is_element) {
        let name = child.tag_name().name();
        if name == record_name {
            index += 1;
            let path = format!("{root_name}/{name}[{index}]");
            if root_name == "RECIPES" {
                document.recipes.push(reader.recipe(child, &path));
            }
        } else if !name.starts_with("DISPLAY_") {
            reader.unknown(format!("{root_name}/{name}"));
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

    fn recipe(&mut self, node: Node<'_, '_>, path: &str) -> Recipe {
        let mut recipe = Recipe::default();
        let mut has_name = false;
        for field in node.children().filter(Node::is_element) {
            let name = field.tag_name().name();
            let field_path = format!("{path}/{name}");
            let text = field.text().unwrap_or("");
            match name {
                "NAME" => {
                    text.trim().clone_into(&mut recipe.name);
                    has_name = !recipe.name.is_empty();
                }
                "BATCH_SIZE" => recipe.batch_size = self.number(text, &field_path),
                "BOIL_SIZE" => recipe.boil_size = self.number(text, &field_path),
                "BOIL_TIME" => recipe.boil_time = self.number(text, &field_path),
                "EFFICIENCY" => recipe.efficiency = self.number(text, &field_path),
                "OG" => recipe.og = self.number(text, &field_path),
                "FG" => recipe.fg = self.number(text, &field_path),
                _ if name.starts_with("DISPLAY_") || RECIPE_ELEMENTS.contains(&name) => {}
                _ => self.unknown(field_path),
            }
        }
        if !has_name {
            self.warn(format!("{path}/NAME"), WarningKind::MissingName);
        }
        recipe
    }

    fn number(&mut self, text: &str, path: &str) -> Option<f64> {
        match leading(text) {
            Leading::Empty => None,
            Leading::Number { value, trailing } => {
                if trailing {
                    self.warn(
                        path,
                        WarningKind::TrailingText {
                            text: text.to_owned(),
                        },
                    );
                }
                Some(value)
            }
            Leading::Unreadable => {
                self.warn(
                    path,
                    WarningKind::UnreadableNumber {
                        text: text.to_owned(),
                    },
                );
                None
            }
        }
    }
}
