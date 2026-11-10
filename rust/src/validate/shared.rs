//! What the 1.0 and 1.1 schemas share: field constructors and the spec's
//! enumerations, which are the same in both XSDs.

use super::{Field, Kind};

pub(super) const fn req(name: &'static str, kind: Kind) -> Field {
    Field {
        name,
        required: true,
        kind,
    }
}

pub(super) const fn opt(name: &'static str, kind: Kind) -> Field {
    Field {
        name,
        required: false,
        kind,
    }
}

pub(super) const HOP_USE: &[&str] = &["Boil", "Dry Hop", "Mash", "First Wort", "Aroma"];
pub(super) const HOP_TYPE: &[&str] = &["Bittering", "Aroma", "Both"];
pub(super) const HOP_FORM: &[&str] = &["Pellet", "Plug", "Leaf"];
pub(super) const FERMENTABLE_TYPE: &[&str] =
    &["Grain", "Sugar", "Extract", "Dry Extract", "Adjunct"];
pub(super) const YEAST_TYPE: &[&str] = &["Ale", "Lager", "Wheat", "Wine", "Champagne"];
pub(super) const YEAST_FORM: &[&str] = &["Liquid", "Dry", "Slant", "Culture"];
pub(super) const YEAST_FLOCCULATION: &[&str] = &["Low", "Medium", "High", "Very High"];
pub(super) const MISC_TYPE: &[&str] =
    &["Spice", "Fining", "Water Agent", "Herb", "Flavor", "Other"];
pub(super) const MISC_USE: &[&str] = &["Boil", "Mash", "Primary", "Secondary", "Bottling"];
pub(super) const STYLE_TYPE: &[&str] = &["Lager", "Ale", "Mead", "Wheat", "Mixed", "Cider"];
pub(super) const MASH_STEP_TYPE: &[&str] = &["Infusion", "Temperature", "Decoction"];
pub(super) const RECIPE_TYPE: &[&str] = &["Extract", "Partial Mash", "All Grain"];
pub(super) const IBU_METHOD: &[&str] = &["Rager", "Tinseth", "Garetz"];
