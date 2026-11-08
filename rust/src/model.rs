use crate::date::RecordDate;
use crate::enums::{
    FermentableType, HopForm, HopType, HopUse, MiscType, MiscUse, YeastFlocculation, YeastForm,
    YeastType,
};
use crate::units::{
    Amount, Celsius, Kilograms, Lintner, Litres, Minutes, Percent, Ppm, SpecificGravity, Srm,
};

/// A BeerXML document: the records of its top-level list.
///
/// BeerXML files hold a single top-level list, so at most one of these is
/// non-empty. Record types that are not modelled yet are accepted (the file
/// still reads) but their records are not returned.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Document {
    /// Recipes from a `RECIPES` list, in document order.
    pub recipes: Vec<Recipe>,
    /// Hops from a top-level `HOPS` list (an ingredient library), in
    /// document order. Hops used by a recipe are on [`Recipe::hops`].
    pub hops: Vec<Hop>,
    /// Fermentables from a top-level `FERMENTABLES` list, in document order.
    /// Those used by a recipe are on [`Recipe::fermentables`].
    pub fermentables: Vec<Fermentable>,
    /// Yeasts from a top-level `YEASTS` list, in document order. Those used
    /// by a recipe are on [`Recipe::yeasts`].
    pub yeasts: Vec<Yeast>,
    /// Miscs from a top-level `MISCS` list, in document order. Those used by
    /// a recipe are on [`Recipe::miscs`].
    pub miscs: Vec<Misc>,
    /// Water profiles from a top-level `WATERS` list, in document order.
    /// Those used by a recipe are on [`Recipe::waters`].
    pub waters: Vec<Water>,
}

/// A `RECIPE` record.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Recipe {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `BATCH_SIZE`: target size of the finished batch.
    pub batch_size: Option<Litres>,
    /// `BOIL_SIZE`: starting pre-boil volume.
    pub boil_size: Option<Litres>,
    /// `BOIL_TIME`: boil length.
    pub boil_time: Option<Minutes>,
    /// `EFFICIENCY`: brewhouse efficiency.
    pub efficiency: Option<Percent>,
    /// `OG`: measured original gravity.
    pub og: Option<SpecificGravity>,
    /// `FG`: measured final gravity.
    pub fg: Option<SpecificGravity>,
    /// `HOPS`: the hop additions, in document order.
    pub hops: Vec<Hop>,
    /// `FERMENTABLES`: the fermentable additions, in document order.
    pub fermentables: Vec<Fermentable>,
    /// `YEASTS`: the yeasts pitched, in document order.
    pub yeasts: Vec<Yeast>,
    /// `MISCS`: the misc additions, in document order.
    pub miscs: Vec<Misc>,
    /// `WATERS`: the water profiles used, in document order.
    pub waters: Vec<Water>,
}

/// A `HOP` record: a hop variety, or a hop addition when inside a recipe.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Hop {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `ALPHA`: alpha acid content.
    pub alpha: Option<Percent>,
    /// `AMOUNT`: weight of the addition. Hop amounts are always weights.
    pub amount: Option<Kilograms>,
    /// `USE`: when the hop is added.
    pub usage: Option<HopUse>,
    /// `TIME`: how long the hop is used; its meaning depends on
    /// [`usage`](Self::usage), e.g. boil time for `Boil`, contact time for
    /// `Dry Hop`.
    pub time: Option<Minutes>,
    /// `NOTES`.
    pub notes: Option<String>,
    /// `TYPE`: bittering, aroma or both.
    pub kind: Option<HopType>,
    /// `FORM`: pellet, plug or leaf.
    pub form: Option<HopForm>,
    /// `BETA`: beta acid content.
    pub beta: Option<Percent>,
    /// `HSI`: hop stability index, the percentage of alpha acid lost after
    /// six months at room temperature.
    pub hsi: Option<Percent>,
    /// `ORIGIN`: where the hop was grown.
    pub origin: Option<String>,
    /// `SUBSTITUTES`: suggested substitutes.
    pub substitutes: Option<String>,
    /// `HUMULENE`: humulene content.
    pub humulene: Option<Percent>,
    /// `CARYOPHYLLENE`: caryophyllene content.
    pub caryophyllene: Option<Percent>,
    /// `COHUMULONE`: cohumulone content.
    pub cohumulone: Option<Percent>,
    /// `MYRCENE`: myrcene content.
    pub myrcene: Option<Percent>,
}

/// A `FERMENTABLE` record: a grain, sugar or extract, or an addition of one
/// when inside a recipe.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Fermentable {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `TYPE`: grain, sugar, extract, dry extract or adjunct.
    pub kind: Option<FermentableType>,
    /// `AMOUNT`: weight of the addition.
    pub amount: Option<Kilograms>,
    /// `YIELD`: percent dry yield (fine grain) relative to sucrose.
    pub yield_: Option<Percent>,
    /// `COLOR`: the colour of the ingredient.
    pub color: Option<Srm>,
    /// `ADD_AFTER_BOIL`: whether it is added after the boil.
    pub add_after_boil: Option<bool>,
    /// `ORIGIN`: where it comes from.
    pub origin: Option<String>,
    /// `SUPPLIER`: who supplies it.
    pub supplier: Option<String>,
    /// `NOTES`.
    pub notes: Option<String>,
    /// `COARSE_FINE_DIFF`: yield difference between coarse and fine grind.
    pub coarse_fine_diff: Option<Percent>,
    /// `MOISTURE`: moisture content.
    pub moisture: Option<Percent>,
    /// `DIASTATIC_POWER`: enzymatic power. The 1.1 schema also lists the
    /// misspelling `DISASTATIC_POWER`, which is read into this field too.
    pub diastatic_power: Option<Lintner>,
    /// `PROTEIN`: protein content.
    pub protein: Option<Percent>,
    /// `MAX_IN_BATCH`: the recommended maximum share of the grain bill.
    pub max_in_batch: Option<Percent>,
    /// `RECOMMEND_MASH`: whether the ingredient needs mashing.
    pub recommend_mash: Option<bool>,
    /// `IBU_GAL_PER_LB`: for hopped extracts, the bitterness a pound adds to
    /// a gallon of wort, in IBU · gal / lb.
    pub ibu_gal_per_lb: Option<f64>,
    /// `POTENTIAL`: the gravity a pound yields in a gallon of water. Not in
    /// the 1.0 spec, but written by BeerSmith and listed in the 1.1 schema.
    pub potential: Option<SpecificGravity>,
}

/// A `YEAST` record: a yeast strain, or a pitch of one when inside a recipe.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Yeast {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `TYPE`: ale, lager, wheat, wine or champagne.
    pub kind: Option<YeastType>,
    /// `FORM`: liquid, dry, slant or culture.
    pub form: Option<YeastForm>,
    /// `AMOUNT`, a weight or a volume as `AMOUNT_IS_WEIGHT` says. The spec
    /// defaults to a volume when `AMOUNT_IS_WEIGHT` is absent.
    pub amount: Option<Amount>,
    /// `LABORATORY`: who produces the yeast.
    pub laboratory: Option<String>,
    /// `PRODUCT_ID`: the laboratory's product code.
    pub product_id: Option<String>,
    /// `MIN_TEMPERATURE`: lowest recommended fermentation temperature.
    pub min_temperature: Option<Celsius>,
    /// `MAX_TEMPERATURE`: highest recommended fermentation temperature.
    pub max_temperature: Option<Celsius>,
    /// `FLOCCULATION`: how well the yeast settles.
    pub flocculation: Option<YeastFlocculation>,
    /// `ATTENUATION`: typical apparent attenuation.
    pub attenuation: Option<Percent>,
    /// `NOTES`.
    pub notes: Option<String>,
    /// `BEST_FOR`: styles the yeast suits.
    pub best_for: Option<String>,
    /// `TIMES_CULTURED`: how often the sample has been recultured.
    pub times_cultured: Option<u32>,
    /// `MAX_REUSE`: recommended maximum number of reuses.
    pub max_reuse: Option<u32>,
    /// `ADD_TO_SECONDARY`: whether it is pitched into the secondary.
    pub add_to_secondary: Option<bool>,
    /// `CULTURE_DATE`: when the sample was cultured. Not in the 1.0 spec,
    /// but written by BeerSmith and listed in the 1.1 schema.
    pub culture_date: Option<RecordDate>,
}

/// A `MISC` record: a spice, fining, water agent, herb or flavouring, or an
/// addition of one when inside a recipe.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Misc {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `TYPE`: what kind of ingredient it is.
    pub kind: Option<MiscType>,
    /// `USE`: when it is added.
    pub usage: Option<MiscUse>,
    /// `TIME`: how long it is used, e.g. boil time for `Boil`.
    pub time: Option<Minutes>,
    /// `AMOUNT`, a weight or a volume as `AMOUNT_IS_WEIGHT` says. The spec
    /// defaults to a volume when `AMOUNT_IS_WEIGHT` is absent.
    pub amount: Option<Amount>,
    /// `USE_FOR`: what it is used for, e.g. `Clarity`.
    pub use_for: Option<String>,
    /// `NOTES`.
    pub notes: Option<String>,
}

/// A `WATER` record: a water profile.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Water {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `AMOUNT`: volume of water used.
    pub amount: Option<Litres>,
    /// `CALCIUM`: calcium (Ca).
    pub calcium: Option<Ppm>,
    /// `BICARBONATE`: bicarbonate (HCO₃).
    pub bicarbonate: Option<Ppm>,
    /// `SULFATE`: sulfate (SO₄).
    pub sulfate: Option<Ppm>,
    /// `CHLORIDE`: chloride (Cl).
    pub chloride: Option<Ppm>,
    /// `SODIUM`: sodium (Na).
    pub sodium: Option<Ppm>,
    /// `MAGNESIUM`: magnesium (Mg).
    pub magnesium: Option<Ppm>,
    /// `PH`: the water's pH.
    pub ph: Option<f64>,
    /// `NOTES`.
    pub notes: Option<String>,
}
