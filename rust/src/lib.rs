//! Reference implementation for reading BeerXML 1.0 and 1.1.
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
//! # Reading rules
//!
//! Lenient reading follows the rules of the Python reference library in this
//! repository and of the BeerXML 1.0 spec:
//!
//! - **Version:** see [`Version`].
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

#![forbid(unsafe_code)]

mod date;
mod enums;
mod error;
mod model;
mod number;
mod reader;
pub mod units;
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
pub use version::Version;
pub use warning::{Warning, WarningKind};
