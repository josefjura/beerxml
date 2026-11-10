//! Reference implementation for reading and validating BeerXML 1.0 and 1.1.
//!
//! The entry point is [`read`]: it takes the raw bytes of a BeerXML file,
//! detects its [`Version`] and returns a typed [`Document`] together with
//! every repair it made along the way as a [`Warning`].
//!
//! ```
//! let xml = br#"<RECIPES><RECIPE><NAME>Pale Ale</NAME><OG>1.050</OG></RECIPE></RECIPES>"#;
//! let read = beerxml::read(xml)?;
//! assert_eq!(read.version, beerxml::Version::V1_0);
//! assert_eq!(read.document.recipes[0].name, "Pale Ale");
//! assert_eq!(read.document.recipes[0].og.map(|og| og.get()), Some(1.050));
//! # Ok::<(), beerxml::ReadError>(())
//! ```
//!
//! To check a file you wrote yourself, [`validate`] it strictly against the
//! schema of a version instead: it returns every [`Violation`], and none
//! means valid. For 1.1 that includes RFC 01's "Clean XML" rules.
//!
//! ```
//! use beerxml::{Rule, Version};
//!
//! let xml = br#"<HOPS><HOP><NAME>Saaz</NAME><VERSION>1</VERSION><AMOUNT>0.05</AMOUNT>
//!     <USE>Boil</USE><TIME>60</TIME></HOP></HOPS>"#;
//! let violations = beerxml::validate(xml, Version::V1_0)?;
//! assert_eq!(violations[0].path, "HOPS/HOP[1]/ALPHA");
//! assert_eq!(violations[0].rule, Rule::Required);
//! # Ok::<(), beerxml::ReadError>(())
//! ```
//!
//! # Reading rules
//!
//! Lenient reading takes what real BeerXML 1.0 writers produce and repairs
//! it, reporting every repair. Each rule names its source: a section of the
//! [BeerXML 1.0 spec][spec-1.0] (the [original][original-1.0] where this
//! repository's transcription leaves a sentence out), of
//! [RFC 01][rfc-01] (the 1.1 "Clean XML" goals) or of [XML 1.0][xml]. Where
//! the spec says nothing, the rule is this crate's own choice and says so.
//!
//! - **Version:** see [`Version`].
//!   *Source:* [1.0 spec, General][spec-1.0-general] (`VERSION` is `1` or
//!   `1.0`);
//!   [RFC 01 §5, Namespaces & Versioning][rfc-01-versioning] (the 1.1
//!   namespace and `VERSION` 1.1).
//! - **Encoding:** UTF-8 and ISO-8859-1 are read. A UTF-8 byte order mark
//!   means UTF-8. Otherwise the `encoding` in the XML declaration decides,
//!   matched case-insensitively (`ISO-8859-1`, `latin1` and the other IANA
//!   aliases; `US-ASCII` reads as UTF-8). No declaration means UTF-8 (XML 1.0,
//!   section 4.3.3 and Appendix F). Any other encoding, including UTF-16, is
//!   refused with [`ReadError::UnsupportedEncoding`] rather than read as
//!   garbled text.
//!   *Source:* [1.0 spec, General][spec-1.0-general] (ISO-8859-1 and UTF-8);
//!   [RFC 01 §4, Character Encoding][rfc-01-encoding] (1.1 is UTF-8);
//!   [XML 1.0 §4.3.3][xml-encoding] and [Appendix F][xml-autodetect]
//!   (declaration, byte order mark, default). The 1.0 spec makes ISO-8859-1
//!   the default; this crate follows XML 1.0 instead, since an undeclared
//!   ISO-8859-1 file is not well-formed XML.
//! - **Numbers:** the leading decimal number is read, so `30.000000` and
//!   `34.2 IBU` both work. Anything other than whitespace after the number is
//!   reported as [`WarningKind::TrailingText`]. A field with no leading number
//!   is `None` and reported as [`WarningKind::UnreadableNumber`]. An empty
//!   element is treated as absent.
//!   The exception is the 1.0 display fields of the spec's Appendix A on a
//!   recipe (`EST_OG`, `EST_FG`, `EST_COLOR`, `IBU`, `EST_ABV`, `ABV`,
//!   `ACTUAL_EFFICIENCY`, `CALORIES`): writers put units after the number
//!   (`32.4 IBU`), so in 1.0 files that text is not reported.
//!   *Source:* [RFC 01 §1, Data Types & Strictness][rfc-01-strictness] (1.0
//!   numeric fields hold units in practice, e.g. `34.2 IBU`);
//!   [1.0 spec, Appendix A][spec-1.0-appendix-a] (the display fields).
//! - **Units:** quantities are typed by their canonical BeerXML unit (see
//!   [`units`]), so values of different units can't be mixed.
//!   *Source:* [1.0 spec §2, Units][spec-1.0-units];
//!   [RFC 01 §2, Units of Measurement][rfc-01-units].
//! - **Enumerated fields** (e.g. [`HopUse`]) match case- and
//!   whitespace-insensitively. A value outside the spec's list is kept as
//!   `Other` with its original text and reported as
//!   [`WarningKind::UnknownValue`].
//!   *Source:* [1.0 spec, Data Formats][spec-1.0-formats] ("List") and the
//!   values listed in each [record table][spec-1.0-records]; matching
//!   loosely and keeping unknown values is this crate's choice.
//! - **Booleans** are `TRUE` or `FALSE` in any case. Anything else is `None`
//!   and reported as [`WarningKind::UnreadableBoolean`].
//!   *Source:* [1.0 spec, Data Formats][spec-1.0-formats] ("Boolean:
//!   `TRUE` or `FALSE`"); [1.1 spec, Changes from v1.0][spec-1.1-changes]
//!   (lowercase `true`/`false`), hence any case. That section also allows
//!   XML Schema's `1`/`0`, which strict [`validate`] accepts but lenient
//!   reading does not yet: they are reported as unreadable.
//! - **Integers** (e.g. [`Yeast::max_reuse`]) follow the number rule and
//!   must be whole and non-negative; otherwise they are `None` and reported
//!   as [`WarningKind::UnreadableNumber`].
//!   *Source:* [1.0 spec, Data Formats][spec-1.0-formats] ("Integer: Whole
//!   numbers").
//! - **Dates** keep their original text, plus the date when it is ISO
//!   `YYYY-MM-DD` (see [`RecordDate`]).
//!   *Source:* [1.0 spec, Recipe record][spec-1.0-recipe] (`DATE` is
//!   text); [RFC 01 §3, Date Format][rfc-01-dates] (free text in 1.0, ISO
//!   8601 in 1.1).
//! - **Amounts** of Misc and Yeast records are a weight or a volume as their
//!   `AMOUNT_IS_WEIGHT` flag says, wherever the flag sits in the record. An
//!   absent (or unreadable) flag means a volume, the spec's default.
//!   *Source:* [1.0 spec, Yeast][spec-1.0-yeast] and
//!   [Misc][spec-1.0-misc] records; the [original 1.0 spec][original-1.0]
//!   gives `FALSE` as the default.
//! - **Text fields** are trimmed; an empty text field is `None`.
//!   *Source:* this crate's choice.
//! - **Missing optional fields** are `None`, never a made-up default.
//!   *Source:* this crate's choice; the spec's defaults are left to the
//!   application.
//! - **`DISPLAY_*` fields** are ignored: values are always read from the
//!   canonical fields.
//!   *Source:* [1.0 spec, Appendix A][spec-1.0-appendix-a] (display fields
//!   are optional and for display only);
//!   [RFC 01 §2, Units of Measurement][rfc-01-units] and
//!   [1.1 spec, Changes from v1.0][spec-1.1-changes] (removed in 1.1).
//! - **Unknown elements** are ignored. BeerXML 1.0 requires readers to ignore
//!   them silently; in 1.1 each one is reported as
//!   [`WarningKind::UnknownElement`].
//!   *Source:* the [original 1.0 spec][original-1.0], "Non-Standard Tags"
//!   (all non-standard tags are ignored by the importing program);
//!   [the 1.1 XSD][xsd-1.1] (the elements 1.1 allows).
//! - **Records** are returned in document order.
//!   *Source:* this crate's choice.
//!
//! [spec-1.0]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/
//! [spec-1.0-general]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#general
//! [spec-1.0-units]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#units
//! [spec-1.0-formats]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#data-formats
//! [spec-1.0-records]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#3-record-specifications
//! [spec-1.0-yeast]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#yeast-record-yeast
//! [spec-1.0-misc]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#misc-record-misc
//! [spec-1.0-recipe]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#recipe-record-recipe
//! [spec-1.0-appendix-a]: https://josefjura.github.io/beerxml/spec/v1.0/v1.0/#appendix-a-optional-extensions-for-display
//! [original-1.0]: http://www.beerxml.com/beerxml.htm
//! [spec-1.1-changes]: https://josefjura.github.io/beerxml/spec/v1.1/v1.1/#changes-from-v10
//! [xsd-1.1]: https://github.com/josefjura/beerxml/blob/master/docs/spec/v1.1/beerxml-1.1.xsd
//! [rfc-01]: https://josefjura.github.io/beerxml/spec/v1.1/RFC_01_goals/
//! [rfc-01-strictness]: https://josefjura.github.io/beerxml/spec/v1.1/RFC_01_goals/#1-data-types-strictness
//! [rfc-01-units]: https://josefjura.github.io/beerxml/spec/v1.1/RFC_01_goals/#2-units-of-measurement
//! [rfc-01-dates]: https://josefjura.github.io/beerxml/spec/v1.1/RFC_01_goals/#3-date-format
//! [rfc-01-encoding]: https://josefjura.github.io/beerxml/spec/v1.1/RFC_01_goals/#4-character-encoding
//! [rfc-01-versioning]: https://josefjura.github.io/beerxml/spec/v1.1/RFC_01_goals/#5-namespaces-versioning
//! [xml]: https://www.w3.org/TR/xml/
//! [xml-encoding]: https://www.w3.org/TR/xml/#charencoding
//! [xml-autodetect]: https://www.w3.org/TR/xml/#sec-guessing
//!
//! # Untrusted input
//!
//! [`read`] is safe to run on uploads from strangers, e.g. inside a request
//! handler:
//!
//! - **No DTDs.** A document with a `<!DOCTYPE …>` is refused with
//!   [`ReadError::NotXml`]. BeerXML defines no DTD and real exports carry
//!   none, so this costs nothing. Without a DTD no entity can be declared,
//!   which rules out entity-expansion attacks (billion laughs, quadratic
//!   blowup) and external entities. Only XML's predefined entities (`&amp;`,
//!   `&lt;`, …) and character references (`&#xFD;`) are read.
//! - **No I/O.** The crate never opens files or network connections: an
//!   external DTD or `SYSTEM` entity is refused, not fetched. The parser,
//!   [`roxmltree`](https://docs.rs/roxmltree), has no I/O code at all.
//! - **Linear cost.** With nothing to expand, time and memory grow with the
//!   size of the input: the decoded text, the parsed tree (a node per
//!   element, text run, comment and processing instruction, plus its
//!   attributes) and the typed records. The tests read a generated 5 MB
//!   ingredient library within a 10 s budget set for unoptimised builds; a
//!   release build reads it in about a tenth of a second. Cap the upload
//!   size to cap the cost.
//! - **No `unsafe`** code in this crate (`#![forbid(unsafe_code)]`).
//!
//! These guarantees are tested in `tests/hostile_input.rs`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod date;
mod encoding;
mod enums;
mod error;
mod model;
mod number;
mod reader;
pub mod units;
mod validate;
mod version;
mod warning;

pub use date::{IsoDate, RecordDate};
pub use enums::{
    FermentableType, HopForm, HopType, HopUse, IbuMethod, MashStepType, MiscType, MiscUse,
    RecipeType, StyleType, YeastFlocculation, YeastForm, YeastType,
};
pub use error::ReadError;
pub use model::{
    Document, Equipment, Fermentable, Hop, Mash, MashStep, Misc, Recipe, Style, Water, Yeast,
};
pub use reader::{Read, read};
pub use validate::{Rule, Violation, validate};
pub use version::Version;
pub use warning::{Warning, WarningKind};
