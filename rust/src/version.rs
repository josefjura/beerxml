use std::fmt;

/// The BeerXML version a document was detected as.
///
/// Detection mirrors the Python reference validator
/// (`tests/validate_samples.py`) so both implementations treat every file the
/// same way. A document is 1.1 when either:
///
/// - its root element is in a namespace containing `beerxml.com/v1.1`
///   (e.g. `xmlns="http://beerxml.com/v1.1"`), or
/// - the first non-namespaced `VERSION` element in document order has the
///   exact text `1.1`.
///
/// Otherwise it is 1.0.
///
/// *Source:* [1.0 spec, General](https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#general)
/// (`VERSION` is `1`) and
/// [RFC 01 §5, Namespaces & Versioning](https://josefjura.github.io/beerxml/spec/v1.1/RFC_01_goals/#5-namespaces-versioning)
/// (the `http://beerxml.com/v1.1` namespace and `VERSION` 1.1).
///
/// Detection is looser than the 1.1 version marker that strict
/// [`validate`](crate::validate) enforces (the namespace on the root plus
/// `VERSION` 1.1 in every record), so every file that validates as 1.1 is
/// also detected as 1.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Version {
    /// BeerXML 1.0 (1.0 spec, legacy files from BeerSmith, Brewfather, …).
    V1_0,
    /// BeerXML 1.1 "Clean XML" draft (see RFC 01 in this repository).
    V1_1,
}

impl Version {
    pub(crate) fn detect(doc: &roxmltree::Document<'_>) -> Self {
        let root = doc.root_element();
        let namespaced = root
            .tag_name()
            .namespace()
            .is_some_and(|ns| ns.contains("beerxml.com/v1.1"));
        let version_element = root
            .descendants()
            .skip(1)
            .find(|n| {
                n.is_element()
                    && n.tag_name().name() == "VERSION"
                    && n.tag_name().namespace().is_none()
            })
            .is_some_and(|n| n.text() == Some("1.1"));
        if namespaced || version_element {
            Self::V1_1
        } else {
            Self::V1_0
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
        })
    }
}
