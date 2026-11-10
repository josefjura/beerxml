//! The BeerXML 1.0 schema, written by hand from
//! `docs/spec/v1.0/beerxml-1.0.xsd`, in the XSD's order.
//!
//! XSD types map to [`Kind`]s as follows: `xs:string` is text, `xs:int` an
//! integer, `decimalType` and `timeType` non-negative decimals,
//! `temperatureType` and `xs:decimal` decimals, `booleanType` a boolean, and
//! the `*Type` enumerations their lists. Optional fields that are
//! `xs:string` and only for display (`DISPLAY_*`, `INVENTORY`) are left out,
//! since any text is valid in them.

use super::Kind::{
    Boolean, Decimal, Enumeration, Integer, List as ListOf, NonNegativeDecimal as NonNeg, Record,
    Text,
};
use super::{Field, Kind, List};

const fn req(name: &'static str, kind: Kind) -> Field {
    Field {
        name,
        required: true,
        kind,
    }
}

const fn opt(name: &'static str, kind: Kind) -> Field {
    Field {
        name,
        required: false,
        kind,
    }
}

const HOP_USE: &[&str] = &["Boil", "Dry Hop", "Mash", "First Wort", "Aroma"];
const HOP_TYPE: &[&str] = &["Bittering", "Aroma", "Both"];
const HOP_FORM: &[&str] = &["Pellet", "Plug", "Leaf"];
const FERMENTABLE_TYPE: &[&str] = &["Grain", "Sugar", "Extract", "Dry Extract", "Adjunct"];
const YEAST_TYPE: &[&str] = &["Ale", "Lager", "Wheat", "Wine", "Champagne"];
const YEAST_FORM: &[&str] = &["Liquid", "Dry", "Slant", "Culture"];
const YEAST_FLOCCULATION: &[&str] = &["Low", "Medium", "High", "Very High"];
const MISC_TYPE: &[&str] = &["Spice", "Fining", "Water Agent", "Herb", "Flavor", "Other"];
const MISC_USE: &[&str] = &["Boil", "Mash", "Primary", "Secondary", "Bottling"];
const STYLE_TYPE: &[&str] = &["Lager", "Ale", "Mead", "Wheat", "Mixed", "Cider"];
const MASH_STEP_TYPE: &[&str] = &["Infusion", "Temperature", "Decoction"];
const RECIPE_TYPE: &[&str] = &["Extract", "Partial Mash", "All Grain"];
const IBU_METHOD: &[&str] = &["Rager", "Tinseth", "Garetz"];

const HOP: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("ALPHA", NonNeg),
    req("AMOUNT", NonNeg),
    req("USE", Enumeration(HOP_USE)),
    req("TIME", NonNeg),
    opt("NOTES", Text),
    opt("TYPE", Enumeration(HOP_TYPE)),
    opt("FORM", Enumeration(HOP_FORM)),
    opt("BETA", NonNeg),
    opt("HSI", NonNeg),
    opt("ORIGIN", Text),
    opt("SUBSTITUTES", Text),
    opt("HUMULENE", NonNeg),
    opt("CARYOPHYLLENE", NonNeg),
    opt("COHUMULONE", NonNeg),
    opt("MYRCENE", NonNeg),
];

const FERMENTABLE: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("TYPE", Enumeration(FERMENTABLE_TYPE)),
    req("AMOUNT", NonNeg),
    req("YIELD", NonNeg),
    req("COLOR", NonNeg),
    opt("ADD_AFTER_BOIL", Boolean),
    opt("ORIGIN", Text),
    opt("SUPPLIER", Text),
    opt("NOTES", Text),
    // COARSE_FINE_DIFF, MOISTURE, DIASTATIC_POWER and PROTEIN are
    // `xs:string` in the 1.0 XSD, so any text is valid.
    opt("COARSE_FINE_DIFF", Text),
    opt("MOISTURE", Text),
    opt("DIASTATIC_POWER", Text),
    opt("DISASTATIC_POWER", Text),
    opt("PROTEIN", Text),
    opt("MAX_IN_BATCH", NonNeg),
    opt("RECOMMEND_MASH", Boolean),
    opt("IBU_GAL_PER_LB", NonNeg),
    opt("IBU_GAL_PER_POUND", NonNeg),
    opt("POTENTIAL", Decimal),
];

const YEAST: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("TYPE", Enumeration(YEAST_TYPE)),
    req("FORM", Enumeration(YEAST_FORM)),
    req("AMOUNT", NonNeg),
    opt("AMOUNT_IS_WEIGHT", Boolean),
    opt("LABORATORY", Text),
    opt("PRODUCT_ID", Text),
    opt("MIN_TEMPERATURE", Decimal),
    opt("MAX_TEMPERATURE", Decimal),
    opt("FLOCCULATION", Enumeration(YEAST_FLOCCULATION)),
    opt("ATTENUATION", NonNeg),
    opt("NOTES", Text),
    opt("BEST_FOR", Text),
    opt("TIMES_CULTURED", Integer),
    opt("MAX_REUSE", Integer),
    opt("ADD_TO_SECONDARY", Boolean),
    opt("CULTURE_DATE", Text),
];

const MISC: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("TYPE", Enumeration(MISC_TYPE)),
    req("USE", Enumeration(MISC_USE)),
    req("TIME", NonNeg),
    req("AMOUNT", NonNeg),
    opt("AMOUNT_IS_WEIGHT", Boolean),
    opt("USE_FOR", Text),
    opt("NOTES", Text),
];

const WATER: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("AMOUNT", NonNeg),
    opt("CALCIUM", NonNeg),
    opt("BICARBONATE", NonNeg),
    opt("SULFATE", NonNeg),
    opt("CHLORIDE", NonNeg),
    opt("SODIUM", NonNeg),
    opt("MAGNESIUM", NonNeg),
    opt("PH", NonNeg),
    opt("NOTES", Text),
];

const EQUIPMENT: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("BOIL_SIZE", NonNeg),
    req("BATCH_SIZE", NonNeg),
    opt("TUN_VOLUME", NonNeg),
    opt("TUN_WEIGHT", NonNeg),
    opt("TUN_SPECIFIC_HEAT", NonNeg),
    opt("TOP_UP_WATER", NonNeg),
    opt("TRUB_CHILLER_LOSS", NonNeg),
    opt("EVAP_RATE", NonNeg),
    opt("BOIL_TIME", NonNeg),
    opt("CALC_BOIL_VOLUME", Boolean),
    opt("LAUTER_DEADSPACE", NonNeg),
    opt("TOP_UP_KETTLE", NonNeg),
    opt("HOP_UTILIZATION", NonNeg),
    opt("NOTES", Text),
];

const STYLE: &[Field] = &[
    req("NAME", Text),
    req("CATEGORY", Text),
    req("VERSION", Integer),
    req("TYPE", Enumeration(STYLE_TYPE)),
    req("OG_MIN", Decimal),
    req("OG_MAX", Decimal),
    req("FG_MIN", Decimal),
    req("FG_MAX", Decimal),
    req("IBU_MIN", NonNeg),
    req("IBU_MAX", NonNeg),
    req("COLOR_MIN", NonNeg),
    req("COLOR_MAX", NonNeg),
    opt("CATEGORY_NUMBER", Text),
    opt("STYLE_LETTER", Text),
    opt("STYLE_GUIDE", Text),
    opt("CARB_MIN", NonNeg),
    opt("CARB_MAX", NonNeg),
    opt("ABV_MIN", NonNeg),
    opt("ABV_MAX", NonNeg),
    opt("NOTES", Text),
    opt("PROFILE", Text),
    opt("INGREDIENTS", Text),
    opt("EXAMPLES", Text),
];

const MASH_STEP: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("TYPE", Enumeration(MASH_STEP_TYPE)),
    opt("INFUSE_AMOUNT", NonNeg),
    req("STEP_TEMP", Decimal),
    req("STEP_TIME", NonNeg),
    opt("RAMP_TIME", NonNeg),
    opt("END_TEMP", Decimal),
    opt("DESCRIPTION", Text),
    opt("WATER_GRAIN_RATIO", Text),
    opt("DECOCTION_AMT", Text),
    opt("INFUSE_TEMP", Text),
];

const MASH: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("GRAIN_TEMP", Decimal),
    req("MASH_STEPS", ListOf(&MASH_STEPS)),
    opt("NOTES", Text),
    opt("TUN_TEMP", Decimal),
    opt("SPARGE_TEMP", Decimal),
    opt("PH", NonNeg),
    opt("TUN_WEIGHT", NonNeg),
    opt("TUN_SPECIFIC_HEAT", NonNeg),
    opt("EQUIP_ADJUST", Boolean),
];

const RECIPE: &[Field] = &[
    req("NAME", Text),
    req("VERSION", Integer),
    req("TYPE", Enumeration(RECIPE_TYPE)),
    req("BREWER", Text),
    opt("ASST_BREWER", Text),
    req("BATCH_SIZE", NonNeg),
    req("BOIL_SIZE", NonNeg),
    req("BOIL_TIME", NonNeg),
    opt("EFFICIENCY", NonNeg),
    opt("STYLE", Record(STYLE)),
    opt("EQUIPMENT", Record(EQUIPMENT)),
    opt("HOPS", ListOf(&HOPS)),
    opt("FERMENTABLES", ListOf(&FERMENTABLES)),
    opt("MISCS", ListOf(&MISCS)),
    opt("YEASTS", ListOf(&YEASTS)),
    opt("WATERS", ListOf(&WATERS)),
    opt("MASH", Record(MASH)),
    opt("NOTES", Text),
    opt("TASTE_NOTES", Text),
    opt("TASTE_RATING", NonNeg),
    opt("RATING", NonNeg),
    opt("OG", Decimal),
    opt("FG", Decimal),
    opt("FERMENTATION_STAGES", Integer),
    opt("PRIMARY_AGE", NonNeg),
    opt("PRIMARY_TEMP", Decimal),
    opt("SECONDARY_AGE", NonNeg),
    opt("SECONDARY_TEMP", Decimal),
    opt("TERTIARY_AGE", NonNeg),
    opt("TERTIARY_TEMP", Decimal),
    opt("AGE", NonNeg),
    opt("AGE_TEMP", Decimal),
    opt("DATE", Text),
    opt("CARBONATION", NonNeg),
    opt("FORCED_CARBONATION", Boolean),
    opt("PRIMING_SUGAR_NAME", Text),
    opt("CARBONATION_TEMP", Decimal),
    opt("PRIMING_SUGAR_EQUIV", NonNeg),
    opt("KEG_PRIMING_FACTOR", NonNeg),
    // The Appendix A display fields (EST_OG, IBU, ABV, …) are `xs:string`
    // in the 1.0 XSD, since writers put units after the number.
    opt("EST_OG", Text),
    opt("EST_FG", Text),
    opt("EST_COLOR", Text),
    opt("IBU", Text),
    opt("IBUS", Text),
    opt("IBU_METHOD", Enumeration(IBU_METHOD)),
    opt("EST_ABV", Text),
    opt("ABV", Text),
    opt("ACTUAL_EFFICIENCY", Text),
    opt("CALORIES", Text),
    opt("CARBONATION_USED", Text),
];

const fn list(
    name: &'static str,
    record: &'static str,
    fields: &'static [Field],
    non_empty: bool,
) -> List {
    List {
        name,
        record,
        fields,
        non_empty,
    }
}

const HOPS: List = list("HOPS", "HOP", HOP, true);
const FERMENTABLES: List = list("FERMENTABLES", "FERMENTABLE", FERMENTABLE, true);
const YEASTS: List = list("YEASTS", "YEAST", YEAST, true);
const MISCS: List = list("MISCS", "MISC", MISC, true);
const WATERS: List = list("WATERS", "WATER", WATER, false);
const STYLES: List = list("STYLES", "STYLE", STYLE, true);
const EQUIPMENTS: List = list("EQUIPMENTS", "EQUIPMENT", EQUIPMENT, true);
const MASH_STEPS: List = list("MASH_STEPS", "MASH_STEP", MASH_STEP, true);
const MASHS: List = list("MASHS", "MASH", MASH, true);
const RECIPES: List = list("RECIPES", "RECIPE", RECIPE, false);

/// The top-level lists a 1.0 document's root may be.
pub(super) const LISTS: &[List] = &[
    HOPS,
    FERMENTABLES,
    YEASTS,
    MISCS,
    WATERS,
    STYLES,
    MASH_STEPS,
    MASHS,
    RECIPES,
    EQUIPMENTS,
];
