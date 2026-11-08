/// A repair made while reading leniently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// Path to the element the warning is about, e.g.
    /// `RECIPES/RECIPE[2]/BATCH_SIZE`. Records carry a 1-based index among
    /// the records of their list; fields carry none.
    pub path: String,
    /// What was repaired.
    pub kind: WarningKind,
}

/// The kind of repair a [`Warning`] reports.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WarningKind {
    /// A number was read, but non-whitespace text followed it and was
    /// ignored (e.g. `34.2 IBU`).
    TrailingText {
        /// The full original text of the field.
        text: String,
    },
    /// A numeric field had no leading number and was read as missing. An
    /// integer field (e.g. `MAX_REUSE`) that is not exactly a whole,
    /// non-negative number, such as `2.5`, `-1` or `3 times`, is reported
    /// the same way.
    UnreadableNumber {
        /// The full original text of the field.
        text: String,
    },
    /// A boolean field is neither `TRUE` nor `FALSE` (in any case) and was
    /// read as missing.
    UnreadableBoolean {
        /// The full original text of the field.
        text: String,
    },
    /// An enumerated field holds a value outside the spec's list. It was
    /// kept as the enum's `Other` variant.
    UnknownValue {
        /// The full original text of the field.
        text: String,
    },
    /// An element that is not part of the BeerXML 1.1 schema was ignored.
    /// Only reported for 1.1 documents.
    UnknownElement,
    /// A record has no `NAME` (or an empty one); its name was read as an
    /// empty string.
    MissingName,
}
