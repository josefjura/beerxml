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
//! schema instead: it returns every [`Violation`], and none means valid.
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
//! Lenient reading follows the rules of the Python reference library in this
//! repository and of the BeerXML 1.0 spec:
//!
//! - **Version:** see [`Version`].
//! - **Encoding:** UTF-8 and ISO-8859-1 are read. A UTF-8 byte order mark
//!   means UTF-8. Otherwise the `encoding` in the XML declaration decides,
//!   matched case-insensitively (`ISO-8859-1`, `latin1` and the other IANA
//!   aliases; `US-ASCII` reads as UTF-8). No declaration means UTF-8 (XML 1.0,
//!   section 4.3.3 and Appendix F). Any other encoding, including UTF-16, is
//!   refused with [`ReadError::UnsupportedEncoding`] rather than read as
//!   garbled text.
//! - **Numbers:** the leading decimal number is read, so `30.000000` and
//!   `34.2 IBU` both work. Anything other than whitespace after the number is
//!   reported as [`WarningKind::TrailingText`]. A field with no leading number
//!   is `None` and reported as [`WarningKind::UnreadableNumber`]. An empty
//!   element is treated as absent.
//!   The exception is the 1.0 display fields of the spec's Appendix A on a
//!   recipe (`EST_OG`, `EST_FG`, `EST_COLOR`, `IBU`, `EST_ABV`, `ABV`,
//!   `ACTUAL_EFFICIENCY`, `CALORIES`): writers put units after the number
//!   (`32.4 IBU`), so in 1.0 files that text is not reported.
//! - **Units:** quantities are typed by their canonical BeerXML unit (see
//!   [`units`]), so values of different units can't be mixed.
//! - **Enumerated fields** (e.g. [`HopUse`]) match case- and
//!   whitespace-insensitively. A value outside the spec's list is kept as
//!   `Other` with its original text and reported as
//!   [`WarningKind::UnknownValue`].
//! - **Booleans** are `TRUE` or `FALSE` in any case. Anything else is `None`
//!   and reported as [`WarningKind::UnreadableBoolean`].
//! - **Integers** (e.g. [`Yeast::max_reuse`]) follow the number rule and
//!   must be whole and non-negative; otherwise they are `None` and reported
//!   as [`WarningKind::UnreadableNumber`].
//! - **Dates** keep their original text, plus the date when it is ISO
//!   `YYYY-MM-DD` (see [`RecordDate`]).
//! - **Amounts** of Misc and Yeast records are a weight or a volume as their
//!   `AMOUNT_IS_WEIGHT` flag says, wherever the flag sits in the record. An
//!   absent (or unreadable) flag means a volume, the spec's default.
//! - **Text fields** are trimmed; an empty text field is `None`.
//! - **Missing optional fields** are `None`, never a made-up default.
//! - **`DISPLAY_*` fields** are ignored: values are always read from the
//!   canonical fields.
//! - **Unknown elements** are ignored. BeerXML 1.0 requires readers to ignore
//!   them silently; in 1.1 each one is reported as
//!   [`WarningKind::UnknownElement`].
//! - **Records** are returned in document order.
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
