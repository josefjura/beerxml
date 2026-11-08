use crate::date::RecordDate;
use crate::enums::{
    FermentableType, HopForm, HopType, HopUse, IbuMethod, MashStepType, MiscType, MiscUse,
    RecipeType, StyleType, YeastFlocculation, YeastForm, YeastType,
};
use crate::units::{
    Amount, Celsius, Co2Volumes, Days, Ibu, Kilograms, Lintner, Litres, Minutes, Percent, Ppm,
    SpecificGravity, Srm,
};

/// A BeerXML document: the records of its top-level list.
///
/// BeerXML files hold a single top-level list, so at most one of these is
/// non-empty.
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
    /// Styles from a top-level `STYLES` list, in document order. A recipe's
    /// style is [`Recipe::style`].
    pub styles: Vec<Style>,
    /// Equipment profiles from a top-level `EQUIPMENTS` list, in document
    /// order. A recipe's equipment is [`Recipe::equipment`].
    pub equipments: Vec<Equipment>,
    /// Mash profiles from a top-level `MASHS` list, in document order. A
    /// recipe's mash is [`Recipe::mash`].
    pub mashes: Vec<Mash>,
    /// Mash steps from a top-level `MASH_STEPS` list, in document order.
    /// Those of a mash are on [`Mash::mash_steps`].
    pub mash_steps: Vec<MashStep>,
}

/// A `RECIPE` record.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Recipe {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `TYPE`: extract, partial mash or all grain.
    pub kind: Option<RecipeType>,
    /// `STYLE`: the style the recipe is brewed to.
    pub style: Option<Style>,
    /// `EQUIPMENT`: the equipment the recipe is designed for.
    pub equipment: Option<Equipment>,
    /// `BREWER`.
    pub brewer: Option<String>,
    /// `ASST_BREWER`: the assistant brewer.
    pub asst_brewer: Option<String>,
    /// `BATCH_SIZE`: target size of the finished batch.
    pub batch_size: Option<Litres>,
    /// `BOIL_SIZE`: starting pre-boil volume.
    pub boil_size: Option<Litres>,
    /// `BOIL_TIME`: boil length.
    pub boil_time: Option<Minutes>,
    /// `EFFICIENCY`: brewhouse efficiency.
    pub efficiency: Option<Percent>,
    /// `HOPS`: the hop additions, in document order.
    pub hops: Vec<Hop>,
    /// `FERMENTABLES`: the fermentable additions, in document order.
    pub fermentables: Vec<Fermentable>,
    /// `MISCS`: the misc additions, in document order.
    pub miscs: Vec<Misc>,
    /// `YEASTS`: the yeasts pitched, in document order.
    pub yeasts: Vec<Yeast>,
    /// `WATERS`: the water profiles used, in document order.
    pub waters: Vec<Water>,
    /// `MASH`: the mash profile, with its steps.
    pub mash: Option<Mash>,
    /// `NOTES`.
    pub notes: Option<String>,
    /// `TASTE_NOTES`: tasting notes.
    pub taste_notes: Option<String>,
    /// `TASTE_RATING`: a tasting score, e.g. on the 0–50 BJCP scale.
    pub taste_rating: Option<f64>,
    /// `RATING`: a rating. Not in the 1.0 spec, but listed in the 1.1
    /// schema.
    pub rating: Option<f64>,
    /// `OG`: measured original gravity.
    pub og: Option<SpecificGravity>,
    /// `FG`: measured final gravity.
    pub fg: Option<SpecificGravity>,
    /// `FERMENTATION_STAGES`: the number of fermentation stages used.
    pub fermentation_stages: Option<u32>,
    /// `PRIMARY_AGE`: time in primary fermentation.
    pub primary_age: Option<Days>,
    /// `PRIMARY_TEMP`: temperature during primary fermentation.
    pub primary_temp: Option<Celsius>,
    /// `SECONDARY_AGE`: time in secondary fermentation.
    pub secondary_age: Option<Days>,
    /// `SECONDARY_TEMP`: temperature during secondary fermentation.
    pub secondary_temp: Option<Celsius>,
    /// `TERTIARY_AGE`: time in tertiary fermentation.
    pub tertiary_age: Option<Days>,
    /// `TERTIARY_TEMP`: temperature during tertiary fermentation.
    pub tertiary_temp: Option<Celsius>,
    /// `AGE`: time spent ageing in the bottle or keg.
    pub age: Option<Days>,
    /// `AGE_TEMP`: temperature while ageing.
    pub age_temp: Option<Celsius>,
    /// `DATE`: when the recipe was brewed. Free text in 1.0, an ISO date in
    /// 1.1.
    pub date: Option<RecordDate>,
    /// `CARBONATION`: target carbonation.
    pub carbonation: Option<Co2Volumes>,
    /// `FORCED_CARBONATION`: whether the batch is force carbonated.
    pub forced_carbonation: Option<bool>,
    /// `PRIMING_SUGAR_NAME`: the sugar used for priming.
    pub priming_sugar_name: Option<String>,
    /// `CARBONATION_TEMP`: temperature for carbonation or priming.
    pub carbonation_temp: Option<Celsius>,
    /// `PRIMING_SUGAR_EQUIV`: how much of the priming sugar is needed
    /// relative to corn sugar (`1.0` for corn sugar).
    pub priming_sugar_equiv: Option<f64>,
    /// `KEG_PRIMING_FACTOR`: the share of the bottling priming amount used
    /// when priming a keg, e.g. `0.5`.
    pub keg_priming_factor: Option<f64>,
    /// `CARBONATION_USED`: display text describing the carbonation, e.g.
    /// `12 PSI`. Listed in the 1.1 schema.
    pub carbonation_used: Option<String>,
    /// `EST_OG`: estimated original gravity (a 1.0 display extension).
    pub est_og: Option<SpecificGravity>,
    /// `EST_FG`: estimated final gravity (a 1.0 display extension).
    pub est_fg: Option<SpecificGravity>,
    /// `EST_COLOR`: estimated colour (a 1.0 display extension).
    pub est_color: Option<Srm>,
    /// `IBU`: estimated bitterness. The 1.1 schema also lists `IBUS`, which
    /// is read into this field too.
    pub ibu: Option<Ibu>,
    /// `IBU_METHOD`: the formula used for [`ibu`](Self::ibu).
    pub ibu_method: Option<IbuMethod>,
    /// `EST_ABV`: estimated alcohol by volume.
    pub est_abv: Option<Percent>,
    /// `ABV`: measured alcohol by volume.
    pub abv: Option<Percent>,
    /// `ACTUAL_EFFICIENCY`: measured brewhouse efficiency.
    pub actual_efficiency: Option<Percent>,
    /// `CALORIES`: estimated calories. The unit is not standardised;
    /// BeerSmith writes calories per pint (`242 cal/pint`).
    pub calories: Option<f64>,
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

/// A `STYLE` record: a beer style and its ranges.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Style {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `CATEGORY`: the category name, e.g. `English Pale Ale`.
    pub category: Option<String>,
    /// `CATEGORY_NUMBER`: the category number in its guide, e.g. `8` or
    /// `3B`. Text, since guides use letters too.
    pub category_number: Option<String>,
    /// `STYLE_LETTER`: the style letter within its category, e.g. `A`.
    pub style_letter: Option<String>,
    /// `STYLE_GUIDE`: the guide the style is from, e.g. `BJCP 2015`.
    pub style_guide: Option<String>,
    /// `TYPE`: lager, ale, mead, wheat, mixed or cider.
    pub kind: Option<StyleType>,
    /// `OG_MIN`: lowest original gravity.
    pub og_min: Option<SpecificGravity>,
    /// `OG_MAX`: highest original gravity.
    pub og_max: Option<SpecificGravity>,
    /// `FG_MIN`: lowest final gravity.
    pub fg_min: Option<SpecificGravity>,
    /// `FG_MAX`: highest final gravity.
    pub fg_max: Option<SpecificGravity>,
    /// `IBU_MIN`: lowest bitterness.
    pub ibu_min: Option<Ibu>,
    /// `IBU_MAX`: highest bitterness.
    pub ibu_max: Option<Ibu>,
    /// `COLOR_MIN`: lightest colour.
    pub color_min: Option<Srm>,
    /// `COLOR_MAX`: darkest colour.
    pub color_max: Option<Srm>,
    /// `CARB_MIN`: lowest carbonation.
    pub carb_min: Option<Co2Volumes>,
    /// `CARB_MAX`: highest carbonation.
    pub carb_max: Option<Co2Volumes>,
    /// `ABV_MIN`: lowest alcohol by volume.
    pub abv_min: Option<Percent>,
    /// `ABV_MAX`: highest alcohol by volume.
    pub abv_max: Option<Percent>,
    /// `NOTES`: a description of the style.
    pub notes: Option<String>,
    /// `PROFILE`: its flavour and aroma profile.
    pub profile: Option<String>,
    /// `INGREDIENTS`: typical ingredients.
    pub ingredients: Option<String>,
    /// `EXAMPLES`: commercial examples.
    pub examples: Option<String>,
}

/// An `EQUIPMENT` record: a brewing equipment profile.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Equipment {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `BOIL_SIZE`: pre-boil volume.
    pub boil_size: Option<Litres>,
    /// `BATCH_SIZE`: target volume in the fermenter.
    pub batch_size: Option<Litres>,
    /// `TUN_VOLUME`: mash tun volume.
    pub tun_volume: Option<Litres>,
    /// `TUN_WEIGHT`: mash tun weight, for heat calculations.
    pub tun_weight: Option<Kilograms>,
    /// `TUN_SPECIFIC_HEAT`: specific heat of the tun material, in
    /// cal / (g · °C).
    pub tun_specific_heat: Option<f64>,
    /// `TOP_UP_WATER`: water added to the fermenter.
    pub top_up_water: Option<Litres>,
    /// `TRUB_CHILLER_LOSS`: wort lost to trub and the chiller.
    pub trub_chiller_loss: Option<Litres>,
    /// `EVAP_RATE`: evaporation per hour of boil.
    pub evap_rate: Option<Percent>,
    /// `BOIL_TIME`: normal boil length.
    pub boil_time: Option<Minutes>,
    /// `CALC_BOIL_VOLUME`: whether the boil volume is calculated from the
    /// batch size.
    pub calc_boil_volume: Option<bool>,
    /// `LAUTER_DEADSPACE`: wort lost in the lauter tun.
    pub lauter_deadspace: Option<Litres>,
    /// `TOP_UP_KETTLE`: water added to the kettle before the boil.
    pub top_up_kettle: Option<Litres>,
    /// `HOP_UTILIZATION`: hop utilisation relative to normal (`100.0`).
    pub hop_utilization: Option<Percent>,
    /// `NOTES`.
    pub notes: Option<String>,
}

/// A `MASH` record: a mash profile and its steps.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct Mash {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `GRAIN_TEMP`: grain temperature at mash-in.
    pub grain_temp: Option<Celsius>,
    /// `MASH_STEPS`: the steps, in document order.
    pub mash_steps: Vec<MashStep>,
    /// `NOTES`.
    pub notes: Option<String>,
    /// `TUN_TEMP`: tun temperature at mash-in.
    pub tun_temp: Option<Celsius>,
    /// `SPARGE_TEMP`: sparge water temperature.
    pub sparge_temp: Option<Celsius>,
    /// `PH`: target mash pH.
    pub ph: Option<f64>,
    /// `TUN_WEIGHT`: mash tun weight, overriding the equipment's.
    pub tun_weight: Option<Kilograms>,
    /// `TUN_SPECIFIC_HEAT`: specific heat of the tun material, in
    /// cal / (g · °C), overriding the equipment's.
    pub tun_specific_heat: Option<f64>,
    /// `EQUIP_ADJUST`: whether temperatures are adjusted for the tun's
    /// thermal mass.
    pub equip_adjust: Option<bool>,
}

/// A `MASH_STEP` record: one step of a mash profile.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct MashStep {
    /// `NAME`, with surrounding whitespace trimmed. The only required field.
    pub name: String,
    /// `TYPE`: infusion, temperature or decoction.
    pub kind: Option<MashStepType>,
    /// `INFUSE_AMOUNT`: water added, for infusion steps.
    pub infuse_amount: Option<Litres>,
    /// `STEP_TEMP`: target temperature of the step.
    pub step_temp: Option<Celsius>,
    /// `STEP_TIME`: how long the step is held.
    pub step_time: Option<Minutes>,
    /// `RAMP_TIME`: how long it takes to reach the step temperature.
    pub ramp_time: Option<Minutes>,
    /// `END_TEMP`: temperature at the end of the step.
    pub end_temp: Option<Celsius>,
    /// `DESCRIPTION`.
    pub description: Option<String>,
    /// `WATER_GRAIN_RATIO`: display text for the water-to-grain ratio, as
    /// written (e.g. `1.25` or `1.25 qt/lb`). The 1.1 schema types it as
    /// text, since writers include units.
    pub water_grain_ratio: Option<String>,
    /// `DECOCTION_AMT`: display text for the decoction amount, as written
    /// (e.g. `6.68 qt`).
    pub decoction_amt: Option<String>,
    /// `INFUSE_TEMP`: display text for the infusion water temperature, as
    /// written (e.g. `161.4 F`).
    pub infuse_temp: Option<String>,
}
