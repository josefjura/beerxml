//! Enumerated BeerXML fields.
//!
//! Values are matched case-insensitively, ignoring surrounding whitespace and
//! runs of inner whitespace, so `Dry Hop`, `dry hop` and `DRY  HOP ` are all
//! [`HopUse::DryHop`]. A value outside the spec's list is kept as `Other`
//! with its original text, and reported as
//! [`WarningKind::UnknownValue`](crate::WarningKind::UnknownValue), so vendor
//! extensions don't fail the read.

use std::fmt;

/// Whether `text` matches the canonical spelling `canonical`, ignoring case
/// and whitespace differences.
fn same_value(text: &str, canonical: &str) -> bool {
    let mut words = text.split_whitespace();
    let mut expected = canonical.split(' ');
    loop {
        match (words.next(), expected.next()) {
            (None, None) => return true,
            (Some(a), Some(b)) if a.eq_ignore_ascii_case(b) => {}
            _ => return false,
        }
    }
}

/// An enumerated field read leniently.
pub(crate) trait Lenient: Sized {
    /// Matches `text` leniently; unknown values become `Other`.
    fn parse(text: &str) -> Self;
    /// Whether the value is not in the spec's list.
    fn is_other(&self) -> bool;
}

macro_rules! lenient_enum {
    (
        $(#[$doc:meta])*
        $name:ident { $($(#[$vdoc:meta])* $variant:ident => $text:literal),+ $(,)? }
    ) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$vdoc])* #[doc = concat!("`", $text, "`")] $variant,)+
            /// A value not in the spec's list, with its original text.
            Other(String),
        }

        impl Lenient for $name {
            fn parse(text: &str) -> Self {
                $(if same_value(text, $text) {
                    return Self::$variant;
                })+
                Self::Other(text.to_owned())
            }

            fn is_other(&self) -> bool {
                matches!(self, Self::Other(_))
            }
        }

        impl $name {
            /// The spec's spelling of the value, or the original text for
            /// `Other`.
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $text,)+
                    Self::Other(text) => text,
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

lenient_enum!(
    /// `HOP/USE`: when a hop is added.
    HopUse {
        Boil => "Boil",
        DryHop => "Dry Hop",
        Mash => "Mash",
        FirstWort => "First Wort",
        Aroma => "Aroma",
    }
);

lenient_enum!(
    /// `HOP/TYPE`: what a hop is used for.
    HopType {
        Bittering => "Bittering",
        Aroma => "Aroma",
        Both => "Both",
    }
);

lenient_enum!(
    /// `HOP/FORM`: the physical form of a hop.
    HopForm {
        Pellet => "Pellet",
        Plug => "Plug",
        Leaf => "Leaf",
    }
);

#[cfg(test)]
mod tests {
    use super::{HopForm, HopUse, Lenient};

    #[test]
    fn matches_ignoring_case_and_whitespace() {
        for text in ["Dry Hop", "dry hop", "DRY HOP", " Dry  Hop ", "Dry\tHop\n"] {
            assert_eq!(HopUse::parse(text), HopUse::DryHop, "{text:?}");
        }
        assert_eq!(HopForm::parse("pellet"), HopForm::Pellet);
    }

    #[test]
    fn unknown_values_keep_their_text() {
        assert_eq!(
            HopUse::parse("Whirlpool"),
            HopUse::Other("Whirlpool".into())
        );
        assert_eq!(HopUse::parse("DryHop"), HopUse::Other("DryHop".into()));
        assert_eq!(HopUse::parse("Whirlpool").as_str(), "Whirlpool");
    }

    #[test]
    fn display_uses_the_spec_spelling() {
        assert_eq!(HopUse::parse("first  WORT").to_string(), "First Wort");
    }
}
