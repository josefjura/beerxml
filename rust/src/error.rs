/// A fatal problem that stops a file from being read at all.
///
/// Anything less severe is repaired and reported as a
/// [`Warning`](crate::Warning) instead.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// The input is not well-formed XML (or not valid UTF-8).
    #[error("not a valid XML document: {message}")]
    NotXml {
        /// What the XML parser reported.
        message: String,
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
