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

lenient_enum!(
    /// `FERMENTABLE/TYPE`: what kind of fermentable an ingredient is.
    FermentableType {
        Grain => "Grain",
        Sugar => "Sugar",
        Extract => "Extract",
        DryExtract => "Dry Extract",
        Adjunct => "Adjunct",
    }
);

lenient_enum!(
    /// `YEAST/TYPE`: what a yeast is used for.
    YeastType {
        Ale => "Ale",
        Lager => "Lager",
        Wheat => "Wheat",
        Wine => "Wine",
        Champagne => "Champagne",
    }
);

lenient_enum!(
    /// `YEAST/FORM`: the physical form of a yeast.
    YeastForm {
        Liquid => "Liquid",
        Dry => "Dry",
        Slant => "Slant",
        Culture => "Culture",
    }
);

lenient_enum!(
    /// `YEAST/FLOCCULATION`: how well a yeast settles.
    YeastFlocculation {
        Low => "Low",
        Medium => "Medium",
        High => "High",
        VeryHigh => "Very High",
    }
);

lenient_enum!(
    /// `MISC/TYPE`: what kind of ingredient a misc is.
    ///
    /// The spec's own `Other` value is [`MiscType::OtherType`], since
    /// [`MiscType::Other`] holds values outside the spec's list.
    MiscType {
        Spice => "Spice",
        Fining => "Fining",
        WaterAgent => "Water Agent",
        Herb => "Herb",
        Flavor => "Flavor",
        /// The spec's catch-all type.
        OtherType => "Other",
    }
);

lenient_enum!(
    /// `MISC/USE`: when a misc is added.
    MiscUse {
        Boil => "Boil",
        Mash => "Mash",
        Primary => "Primary",
        Secondary => "Secondary",
        Bottling => "Bottling",
    }
);

lenient_enum!(
    /// `STYLE/TYPE`: what kind of beverage a style describes.
    StyleType {
        Lager => "Lager",
        Ale => "Ale",
        Mead => "Mead",
        Wheat => "Wheat",
        Mixed => "Mixed",
        Cider => "Cider",
    }
);

lenient_enum!(
    /// `MASH_STEP/TYPE`: how a mash step reaches its temperature.
    MashStepType {
        /// Hot water is added.
        Infusion => "Infusion",
        /// The mash is heated directly.
        Temperature => "Temperature",
        /// Part of the mash is drawn off, boiled and returned.
        Decoction => "Decoction",
    }
);

lenient_enum!(
    /// `RECIPE/TYPE`: how a recipe gets its fermentables.
    RecipeType {
        Extract => "Extract",
        PartialMash => "Partial Mash",
        AllGrain => "All Grain",
    }
);

lenient_enum!(
    /// `RECIPE/IBU_METHOD`: the formula used for the recipe's estimated IBU.
    IbuMethod {
        Rager => "Rager",
        Tinseth => "Tinseth",
        Garetz => "Garetz",
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
