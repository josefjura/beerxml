/// A fatal problem that stops a file from being read at all.
///
/// Anything less severe is repaired and reported as a
/// [`Warning`](crate::Warning) instead.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// The input is not well-formed XML, or its bytes are not valid in the
    /// encoding it is read as (UTF-8 unless it declares ISO-8859-1).
    #[error("not a valid XML document: {message}")]
    NotXml {
        /// What the XML parser reported.
        message: String,
    },
    /// The document is in a character encoding other than UTF-8 or
    /// ISO-8859-1, so it is refused rather than read as garbled text.
    #[error("unsupported character encoding {encoding:?}: only UTF-8 and ISO-8859-1 are read")]
    UnsupportedEncoding {
        /// The encoding as the XML declaration names it (e.g.
        /// `windows-1250`), or `UTF-16LE`/`UTF-16BE` for a file that starts
        /// with a UTF-16 byte order mark.
        encoding: String,
    },
    /// The document's root element is not one of the BeerXML top-level lists
    /// (`RECIPES`, `HOPS`, `FERMENTABLES`, `YEASTS`, `MISCS`, `WATERS`,
    /// `STYLES`, `EQUIPMENTS`, `MASHS`, `MASH_STEPS`).
    #[error("not a BeerXML document: root element is <{found}>")]
    NoBeerXmlRoot {
        /// The local name of the root element that was found.
        found: String,
    },
}
