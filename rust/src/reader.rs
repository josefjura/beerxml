use roxmltree::Node;

use crate::encoding;
use crate::enums::Lenient;
use crate::number::{Leading, leading};
use crate::units::{
    Amount, Celsius, Co2Volumes, Days, Ibu, Kilograms, Lintner, Litres, Minutes, Percent, Ppm,
    SpecificGravity, SpecificHeat, Srm,
};
use crate::{
    Document, Equipment, Fermentable, Hop, Mash, MashStep, Misc, ReadError, Recipe, RecordDate,
    Style, Version, Warning, WarningKind, Water, Yeast,
};

/// The result of reading a file leniently.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Read {
    /// The records the file contains.
    pub document: Document,
    /// The BeerXML version the file was detected as.
    pub version: Version,
    /// Every repair made while reading, in document order.
    pub warnings: Vec<Warning>,
}

/// Reads a BeerXML 1.0 or 1.1 file leniently.
///
/// Any real-world file that is well-formed XML with a BeerXML root element
/// reads into a [`Document`]; every repair is reported in
/// [`Read::warnings`]. See the [crate-level docs](crate) for the rules.
///
/// # Errors
///
/// Returns a [`ReadError`] only for fatal problems: the input is not XML, it
/// is in an encoding other than UTF-8 or ISO-8859-1, or its root element is
/// not a BeerXML top-level list.
pub fn read(bytes: &[u8]) -> Result<Read, ReadError> {
    let text = encoding::decode(bytes)?;
    let doc = parse(&text)?;

    let root = doc.root_element();
    let list = root.tag_name().name();
    let mut reader = Reader {
        version: Version::detect(&doc),
        warnings: Vec::new(),
    };
    let mut d = Document::default();
    // Each top-level list and the record element it holds.
    match list {
        "RECIPES" => d.recipes = reader.list(root, list, "RECIPE", Reader::recipe),
        "HOPS" => d.hops = reader.list(root, list, "HOP", Reader::hop),
        "FERMENTABLES" => {
            d.fermentables = reader.list(root, list, "FERMENTABLE", Reader::fermentable);
        }
        "YEASTS" => d.yeasts = reader.list(root, list, "YEAST", Reader::yeast),
        "MISCS" => d.miscs = reader.list(root, list, "MISC", Reader::misc),
        "WATERS" => d.waters = reader.list(root, list, "WATER", Reader::water),
        "STYLES" => d.styles = reader.list(root, list, "STYLE", Reader::style),
        "EQUIPMENTS" => d.equipments = reader.list(root, list, "EQUIPMENT", Reader::equipment),
        "MASHS" => d.mashes = reader.list(root, list, "MASH", Reader::mash),
        "MASH_STEPS" => d.mash_steps = reader.list(root, list, "MASH_STEP", Reader::mash_step),
        _ => {
            return Err(ReadError::NoBeerXmlRoot {
                found: list.to_owned(),
            });
        }
    }
    Ok(Read {
        document: d,
        version: reader.version,
        warnings: reader.warnings,
    })
}

/// Parses decoded text into an XML tree, refusing DTDs.
///
/// Refusing every DTD is what keeps hostile uploads harmless (see the crate
/// docs, "Untrusted input"): no entity can be declared, so none can expand or
/// point outside the file. Spelled out rather than left to the parser's
/// default so a dependency bump can't change it.
pub(crate) fn parse(text: &str) -> Result<roxmltree::Document<'_>, ReadError> {
    let options = roxmltree::ParsingOptions {
        allow_dtd: false,
        ..roxmltree::ParsingOptions::default()
    };
    roxmltree::Document::parse_with_options(text, options).map_err(|e| ReadError::NotXml {
        message: e.to_string(),
    })
}

/// Every element a `RECIPE` may contain in the BeerXML 1.1 schema, used to
/// tell fields that aren't modelled (such as `VERSION`) from unknown
/// elements.
const RECIPE_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "TYPE",
    "BREWER",
    "ASST_BREWER",
    "BATCH_SIZE",
    "BOIL_SIZE",
    "BOIL_TIME",
    "EFFICIENCY",
    "STYLE",
    "EQUIPMENT",
    "HOPS",
    "FERMENTABLES",
    "MISCS",
    "YEASTS",
    "WATERS",
    "MASH",
    "NOTES",
    "TASTE_NOTES",
    "TASTE_RATING",
    "RATING",
    "OG",
    "FG",
    "FERMENTATION_STAGES",
    "PRIMARY_AGE",
    "PRIMARY_TEMP",
    "SECONDARY_AGE",
    "SECONDARY_TEMP",
    "TERTIARY_AGE",
    "TERTIARY_TEMP",
    "AGE",
    "AGE_TEMP",
    "DATE",
    "CARBONATION",
    "FORCED_CARBONATION",
    "PRIMING_SUGAR_NAME",
    "CARBONATION_TEMP",
    "PRIMING_SUGAR_EQUIV",
    "KEG_PRIMING_FACTOR",
    "IBU",
    "IBUS",
    "IBU_METHOD",
    "EST_ABV",
    "ABV",
    "ACTUAL_EFFICIENCY",
    "CALORIES",
    "CARBONATION_USED",
];

/// Every element a `HOP` may contain in the BeerXML 1.1 schema.
const HOP_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "ALPHA",
    "AMOUNT",
    "USE",
    "TIME",
    "NOTES",
    "TYPE",
    "FORM",
    "BETA",
    "HSI",
    "ORIGIN",
    "SUBSTITUTES",
    "HUMULENE",
    "CARYOPHYLLENE",
    "COHUMULONE",
    "MYRCENE",
];

/// Every element a `FERMENTABLE` may contain in the BeerXML 1.1 schema.
const FERMENTABLE_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "TYPE",
    "AMOUNT",
    "YIELD",
    "COLOR",
    "ADD_AFTER_BOIL",
    "ORIGIN",
    "SUPPLIER",
    "NOTES",
    "COARSE_FINE_DIFF",
    "MOISTURE",
    "DIASTATIC_POWER",
    "DISASTATIC_POWER",
    "PROTEIN",
    "MAX_IN_BATCH",
    "RECOMMEND_MASH",
    "IBU_GAL_PER_LB",
    "POTENTIAL",
];

/// Every element a `YEAST` may contain in the BeerXML 1.1 schema.
const YEAST_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "TYPE",
    "FORM",
    "AMOUNT",
    "AMOUNT_IS_WEIGHT",
    "LABORATORY",
    "PRODUCT_ID",
    "MIN_TEMPERATURE",
    "MAX_TEMPERATURE",
    "FLOCCULATION",
    "ATTENUATION",
    "NOTES",
    "BEST_FOR",
    "TIMES_CULTURED",
    "MAX_REUSE",
    "ADD_TO_SECONDARY",
    "CULTURE_DATE",
];

/// Every element a `MISC` may contain in the BeerXML 1.1 schema.
const MISC_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "TYPE",
    "USE",
    "TIME",
    "AMOUNT",
    "AMOUNT_IS_WEIGHT",
    "USE_FOR",
    "NOTES",
];

/// Every element a `WATER` may contain in the BeerXML 1.1 schema.
const WATER_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "AMOUNT",
    "CALCIUM",
    "BICARBONATE",
    "SULFATE",
    "CHLORIDE",
    "SODIUM",
    "MAGNESIUM",
    "PH",
    "NOTES",
];

/// Every element a `STYLE` may contain in the BeerXML 1.1 schema.
const STYLE_ELEMENTS: &[&str] = &[
    "NAME",
    "CATEGORY",
    "VERSION",
    "TYPE",
    "OG_MIN",
    "OG_MAX",
    "FG_MIN",
    "FG_MAX",
    "IBU_MIN",
    "IBU_MAX",
    "COLOR_MIN",
    "COLOR_MAX",
    "CATEGORY_NUMBER",
    "STYLE_LETTER",
    "STYLE_GUIDE",
    "CARB_MIN",
    "CARB_MAX",
    "ABV_MIN",
    "ABV_MAX",
    "NOTES",
    "PROFILE",
    "INGREDIENTS",
    "EXAMPLES",
];

/// Every element an `EQUIPMENT` may contain in the BeerXML 1.1 schema.
const EQUIPMENT_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "BOIL_SIZE",
    "BATCH_SIZE",
    "TUN_VOLUME",
    "TUN_WEIGHT",
    "TUN_SPECIFIC_HEAT",
    "TOP_UP_WATER",
    "TRUB_CHILLER_LOSS",
    "EVAP_RATE",
    "BOIL_TIME",
    "CALC_BOIL_VOLUME",
    "LAUTER_DEADSPACE",
    "TOP_UP_KETTLE",
    "HOP_UTILIZATION",
    "NOTES",
];

/// Every element a `MASH` may contain in the BeerXML 1.1 schema.
const MASH_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "GRAIN_TEMP",
    "MASH_STEPS",
    "NOTES",
    "TUN_TEMP",
    "SPARGE_TEMP",
    "PH",
    "TUN_WEIGHT",
    "TUN_SPECIFIC_HEAT",
    "EQUIP_ADJUST",
];

/// Every element a `MASH_STEP` may contain in the BeerXML 1.1 schema.
const MASH_STEP_ELEMENTS: &[&str] = &[
    "NAME",
    "VERSION",
    "TYPE",
    "INFUSE_AMOUNT",
    "STEP_TEMP",
    "STEP_TIME",
    "RAMP_TIME",
    "END_TEMP",
    "DESCRIPTION",
    "WATER_GRAIN_RATIO",
    "DECOCTION_AMT",
    "INFUSE_TEMP",
];

/// A field of a record being read: its element name, text and path.
struct Field<'a> {
    name: &'a str,
    text: &'a str,
    node: Node<'a, 'a>,
    path: String,
}

struct Reader {
    version: Version,
    warnings: Vec<Warning>,
}

impl Reader {
    fn warn(&mut self, path: impl Into<String>, kind: WarningKind) {
        self.warnings.push(Warning {
            path: path.into(),
            kind,
        });
    }

    /// Reports an unknown element; only 1.1 documents report them, since the
    /// 1.0 spec requires readers to ignore unknown elements silently.
    fn unknown(&mut self, path: String) {
        if self.version == Version::V1_1 {
            self.warn(path, WarningKind::UnknownElement);
        }
    }

    /// Reads the `record_name` children of a list element with `read`, in
    /// document order. Each record's path carries its 1-based index. Other
    /// children are reported as unknown.
    fn list<T>(
        &mut self,
        list: Node<'_, '_>,
        list_path: &str,
        record_name: &str,
        mut read: impl FnMut(&mut Self, Node<'_, '_>, &str) -> T,
    ) -> Vec<T> {
        let mut records = Vec::new();
        for child in list.children().filter(Node::is_element) {
            let name = child.tag_name().name();
            if name == record_name {
                let path = format!("{list_path}/{name}[{}]", records.len() + 1);
                records.push(read(self, child, &path));
            } else if !name.starts_with("DISPLAY_") {
                self.unknown(format!("{list_path}/{name}"));
            }
        }
        records
    }

    /// Walks the fields of a record and returns its `NAME`.
    ///
    /// `field` handles a modelled field and returns `true`. Unhandled fields
    /// are skipped when they are `DISPLAY_*` or in `known` (part of the
    /// schema but not modelled, such as `VERSION`), and reported as unknown
    /// otherwise.
    fn record(
        &mut self,
        node: Node<'_, '_>,
        path: &str,
        known: &[&str],
        mut field: impl FnMut(&mut Self, &Field<'_>) -> bool,
    ) -> String {
        let mut name = String::new();
        for child in node.children().filter(Node::is_element) {
            let f = Field {
                name: child.tag_name().name(),
                text: child.text().unwrap_or(""),
                node: child,
                path: format!("{path}/{}", child.tag_name().name()),
            };
            if f.name == "NAME" {
                f.text.trim().clone_into(&mut name);
            } else if !field(self, &f)
                && !f.name.starts_with("DISPLAY_")
                && !known.contains(&f.name)
            {
                self.unknown(f.path);
            }
        }
        if name.is_empty() {
            self.warn(format!("{path}/NAME"), WarningKind::MissingName);
        }
        name
    }

    fn recipe(&mut self, node: Node<'_, '_>, path: &str) -> Recipe {
        let mut r = Recipe::default();
        // `IBUS` is the 1.1 schema's alias; `IBU` wins when both are present.
        let mut ibus = None;
        r.name = self.record(node, path, RECIPE_ELEMENTS, |s, f| {
            match f.name {
                "TYPE" => r.kind = s.enumerated(f),
                "STYLE" => r.style = Some(s.style(f.node, &f.path)),
                "EQUIPMENT" => r.equipment = Some(s.equipment(f.node, &f.path)),
                "BREWER" => r.brewer = text(f),
                "ASST_BREWER" => r.asst_brewer = text(f),
                "BATCH_SIZE" => r.batch_size = s.number(f).map(Litres::new),
                "BOIL_SIZE" => r.boil_size = s.number(f).map(Litres::new),
                "BOIL_TIME" => r.boil_time = s.number(f).map(Minutes::new),
                "EFFICIENCY" => r.efficiency = s.number(f).map(Percent::new),
                "HOPS" => r.hops = s.list(f.node, &f.path, "HOP", Self::hop),
                "FERMENTABLES" => {
                    r.fermentables = s.list(f.node, &f.path, "FERMENTABLE", Self::fermentable);
                }
                "MISCS" => r.miscs = s.list(f.node, &f.path, "MISC", Self::misc),
                "YEASTS" => r.yeasts = s.list(f.node, &f.path, "YEAST", Self::yeast),
                "WATERS" => r.waters = s.list(f.node, &f.path, "WATER", Self::water),
                "MASH" => r.mash = Some(s.mash(f.node, &f.path)),
                "NOTES" => r.notes = text(f),
                "TASTE_NOTES" => r.taste_notes = text(f),
                "TASTE_RATING" => r.taste_rating = s.number(f),
                "RATING" => r.rating = s.number(f),
                "OG" => r.og = s.number(f).map(SpecificGravity::new),
                "FG" => r.fg = s.number(f).map(SpecificGravity::new),
                "FERMENTATION_STAGES" => r.fermentation_stages = s.integer(f),
                "PRIMARY_AGE" => r.primary_age = s.number(f).map(Days::new),
                "PRIMARY_TEMP" => r.primary_temp = s.number(f).map(Celsius::new),
                "SECONDARY_AGE" => r.secondary_age = s.number(f).map(Days::new),
                "SECONDARY_TEMP" => r.secondary_temp = s.number(f).map(Celsius::new),
                "TERTIARY_AGE" => r.tertiary_age = s.number(f).map(Days::new),
                "TERTIARY_TEMP" => r.tertiary_temp = s.number(f).map(Celsius::new),
                "AGE" => r.age = s.number(f).map(Days::new),
                "AGE_TEMP" => r.age_temp = s.number(f).map(Celsius::new),
                "DATE" => r.date = date(f),
                "CARBONATION" => r.carbonation = s.number(f).map(Co2Volumes::new),
                "FORCED_CARBONATION" => r.forced_carbonation = s.boolean(f),
                "PRIMING_SUGAR_NAME" => r.priming_sugar_name = text(f),
                "CARBONATION_TEMP" => r.carbonation_temp = s.number(f).map(Celsius::new),
                "PRIMING_SUGAR_EQUIV" => r.priming_sugar_equiv = s.number(f),
                "KEG_PRIMING_FACTOR" => r.keg_priming_factor = s.number(f),
                "CARBONATION_USED" => r.carbonation_used = text(f),
                "EST_OG" => r.est_og = s.display_number(f).map(SpecificGravity::new),
                "EST_FG" => r.est_fg = s.display_number(f).map(SpecificGravity::new),
                "EST_COLOR" => r.est_color = s.display_number(f).map(Srm::new),
                "IBU" => r.ibu = s.display_number(f).map(Ibu::new),
                "IBUS" => ibus = s.number(f).map(Ibu::new),
                "IBU_METHOD" => r.ibu_method = s.enumerated(f),
                "EST_ABV" => r.est_abv = s.display_number(f).map(Percent::new),
                "ABV" => r.abv = s.display_number(f).map(Percent::new),
                "ACTUAL_EFFICIENCY" => r.actual_efficiency = s.display_number(f).map(Percent::new),
                "CALORIES" => r.calories = s.display_number(f),
                _ => return false,
            }
            true
        });
        r.ibu = r.ibu.or(ibus);
        r
    }

    fn style(&mut self, node: Node<'_, '_>, path: &str) -> Style {
        let mut sty = Style::default();
        sty.name = self.record(node, path, STYLE_ELEMENTS, |s, f| {
            match f.name {
                "CATEGORY" => sty.category = text(f),
                "CATEGORY_NUMBER" => sty.category_number = text(f),
                "STYLE_LETTER" => sty.style_letter = text(f),
                "STYLE_GUIDE" => sty.style_guide = text(f),
                "TYPE" => sty.kind = s.enumerated(f),
                "OG_MIN" => sty.og_min = s.number(f).map(SpecificGravity::new),
                "OG_MAX" => sty.og_max = s.number(f).map(SpecificGravity::new),
                "FG_MIN" => sty.fg_min = s.number(f).map(SpecificGravity::new),
                "FG_MAX" => sty.fg_max = s.number(f).map(SpecificGravity::new),
                "IBU_MIN" => sty.ibu_min = s.number(f).map(Ibu::new),
                "IBU_MAX" => sty.ibu_max = s.number(f).map(Ibu::new),
                "COLOR_MIN" => sty.color_min = s.number(f).map(Srm::new),
                "COLOR_MAX" => sty.color_max = s.number(f).map(Srm::new),
                "CARB_MIN" => sty.carb_min = s.number(f).map(Co2Volumes::new),
                "CARB_MAX" => sty.carb_max = s.number(f).map(Co2Volumes::new),
                "ABV_MIN" => sty.abv_min = s.number(f).map(Percent::new),
                "ABV_MAX" => sty.abv_max = s.number(f).map(Percent::new),
                "NOTES" => sty.notes = text(f),
                "PROFILE" => sty.profile = text(f),
                "INGREDIENTS" => sty.ingredients = text(f),
                "EXAMPLES" => sty.examples = text(f),
                _ => return false,
            }
            true
        });
        sty
    }

    fn equipment(&mut self, node: Node<'_, '_>, path: &str) -> Equipment {
        let mut e = Equipment::default();
        e.name = self.record(node, path, EQUIPMENT_ELEMENTS, |s, f| {
            match f.name {
                "BOIL_SIZE" => e.boil_size = s.number(f).map(Litres::new),
                "BATCH_SIZE" => e.batch_size = s.number(f).map(Litres::new),
                "TUN_VOLUME" => e.tun_volume = s.number(f).map(Litres::new),
                "TUN_WEIGHT" => e.tun_weight = s.number(f).map(Kilograms::new),
                "TUN_SPECIFIC_HEAT" => e.tun_specific_heat = s.number(f).map(SpecificHeat::new),
                "TOP_UP_WATER" => e.top_up_water = s.number(f).map(Litres::new),
                "TRUB_CHILLER_LOSS" => e.trub_chiller_loss = s.number(f).map(Litres::new),
                "EVAP_RATE" => e.evap_rate = s.number(f).map(Percent::new),
                "BOIL_TIME" => e.boil_time = s.number(f).map(Minutes::new),
                "CALC_BOIL_VOLUME" => e.calc_boil_volume = s.boolean(f),
                "LAUTER_DEADSPACE" => e.lauter_deadspace = s.number(f).map(Litres::new),
                "TOP_UP_KETTLE" => e.top_up_kettle = s.number(f).map(Litres::new),
                "HOP_UTILIZATION" => e.hop_utilization = s.number(f).map(Percent::new),
                "NOTES" => e.notes = text(f),
                _ => return false,
            }
            true
        });
        e
    }

    fn mash(&mut self, node: Node<'_, '_>, path: &str) -> Mash {
        let mut m = Mash::default();
        m.name = self.record(node, path, MASH_ELEMENTS, |s, f| {
            match f.name {
                "GRAIN_TEMP" => m.grain_temp = s.number(f).map(Celsius::new),
                "MASH_STEPS" => {
                    m.mash_steps = s.list(f.node, &f.path, "MASH_STEP", Self::mash_step);
                }
                "NOTES" => m.notes = text(f),
                "TUN_TEMP" => m.tun_temp = s.number(f).map(Celsius::new),
                "SPARGE_TEMP" => m.sparge_temp = s.number(f).map(Celsius::new),
                "PH" => m.ph = s.number(f),
                "TUN_WEIGHT" => m.tun_weight = s.number(f).map(Kilograms::new),
                "TUN_SPECIFIC_HEAT" => m.tun_specific_heat = s.number(f).map(SpecificHeat::new),
                "EQUIP_ADJUST" => m.equip_adjust = s.boolean(f),
                _ => return false,
            }
            true
        });
        m
    }

    fn mash_step(&mut self, node: Node<'_, '_>, path: &str) -> MashStep {
        let mut step = MashStep::default();
        step.name = self.record(node, path, MASH_STEP_ELEMENTS, |s, f| {
            match f.name {
                "TYPE" => step.kind = s.enumerated(f),
                "INFUSE_AMOUNT" => step.infuse_amount = s.number(f).map(Litres::new),
                "STEP_TEMP" => step.step_temp = s.number(f).map(Celsius::new),
                "STEP_TIME" => step.step_time = s.number(f).map(Minutes::new),
                "RAMP_TIME" => step.ramp_time = s.number(f).map(Minutes::new),
                "END_TEMP" => step.end_temp = s.number(f).map(Celsius::new),
                "DESCRIPTION" => step.description = text(f),
                "WATER_GRAIN_RATIO" => step.water_grain_ratio = text(f),
                "DECOCTION_AMT" => step.decoction_amt = text(f),
                "INFUSE_TEMP" => step.infuse_temp = text(f),
                _ => return false,
            }
            true
        });
        step
    }

    fn hop(&mut self, node: Node<'_, '_>, path: &str) -> Hop {
        let mut h = Hop::default();
        h.name = self.record(node, path, HOP_ELEMENTS, |s, f| {
            match f.name {
                "ALPHA" => h.alpha = s.number(f).map(Percent::new),
                "AMOUNT" => h.amount = s.number(f).map(Kilograms::new),
                "USE" => h.usage = s.enumerated(f),
                "TIME" => h.time = s.number(f).map(Minutes::new),
                "NOTES" => h.notes = text(f),
                "TYPE" => h.kind = s.enumerated(f),
                "FORM" => h.form = s.enumerated(f),
                "BETA" => h.beta = s.number(f).map(Percent::new),
                "HSI" => h.hsi = s.number(f).map(Percent::new),
                "ORIGIN" => h.origin = text(f),
                "SUBSTITUTES" => h.substitutes = text(f),
                "HUMULENE" => h.humulene = s.number(f).map(Percent::new),
                "CARYOPHYLLENE" => h.caryophyllene = s.number(f).map(Percent::new),
                "COHUMULONE" => h.cohumulone = s.number(f).map(Percent::new),
                "MYRCENE" => h.myrcene = s.number(f).map(Percent::new),
                _ => return false,
            }
            true
        });
        h
    }

    fn fermentable(&mut self, node: Node<'_, '_>, path: &str) -> Fermentable {
        let mut ferm = Fermentable::default();
        ferm.name = self.record(node, path, FERMENTABLE_ELEMENTS, |s, f| {
            match f.name {
                "TYPE" => ferm.kind = s.enumerated(f),
                "AMOUNT" => ferm.amount = s.number(f).map(Kilograms::new),
                "YIELD" => ferm.yield_ = s.number(f).map(Percent::new),
                "COLOR" => ferm.color = s.number(f).map(Srm::new),
                "ADD_AFTER_BOIL" => ferm.add_after_boil = s.boolean(f),
                "ORIGIN" => ferm.origin = text(f),
                "SUPPLIER" => ferm.supplier = text(f),
                "NOTES" => ferm.notes = text(f),
                "COARSE_FINE_DIFF" => ferm.coarse_fine_diff = s.number(f).map(Percent::new),
                "MOISTURE" => ferm.moisture = s.number(f).map(Percent::new),
                "DIASTATIC_POWER" | "DISASTATIC_POWER" => {
                    ferm.diastatic_power = s.number(f).map(Lintner::new);
                }
                "PROTEIN" => ferm.protein = s.number(f).map(Percent::new),
                "MAX_IN_BATCH" => ferm.max_in_batch = s.number(f).map(Percent::new),
                "RECOMMEND_MASH" => ferm.recommend_mash = s.boolean(f),
                "IBU_GAL_PER_LB" => ferm.ibu_gal_per_lb = s.number(f),
                "POTENTIAL" => ferm.potential = s.number(f).map(SpecificGravity::new),
                _ => return false,
            }
            true
        });
        ferm
    }

    fn yeast(&mut self, node: Node<'_, '_>, path: &str) -> Yeast {
        let mut y = Yeast::default();
        let mut amount = AmountFields::default();
        y.name = self.record(node, path, YEAST_ELEMENTS, |s, f| {
            match f.name {
                "TYPE" => y.kind = s.enumerated(f),
                "FORM" => y.form = s.enumerated(f),
                "AMOUNT" => amount.value = s.number(f),
                "AMOUNT_IS_WEIGHT" => amount.is_weight = s.boolean(f),
                "LABORATORY" => y.laboratory = text(f),
                "PRODUCT_ID" => y.product_id = text(f),
                "MIN_TEMPERATURE" => y.min_temperature = s.number(f).map(Celsius::new),
                "MAX_TEMPERATURE" => y.max_temperature = s.number(f).map(Celsius::new),
                "FLOCCULATION" => y.flocculation = s.enumerated(f),
                "ATTENUATION" => y.attenuation = s.number(f).map(Percent::new),
                "NOTES" => y.notes = text(f),
                "BEST_FOR" => y.best_for = text(f),
                "TIMES_CULTURED" => y.times_cultured = s.integer(f),
                "MAX_REUSE" => y.max_reuse = s.integer(f),
                "ADD_TO_SECONDARY" => y.add_to_secondary = s.boolean(f),
                "CULTURE_DATE" => y.culture_date = date(f),
                _ => return false,
            }
            true
        });
        y.amount = amount.resolve();
        y
    }

    fn misc(&mut self, node: Node<'_, '_>, path: &str) -> Misc {
        let mut m = Misc::default();
        let mut amount = AmountFields::default();
        m.name = self.record(node, path, MISC_ELEMENTS, |s, f| {
            match f.name {
                "TYPE" => m.kind = s.enumerated(f),
                "USE" => m.usage = s.enumerated(f),
                "TIME" => m.time = s.number(f).map(Minutes::new),
                "AMOUNT" => amount.value = s.number(f),
                "AMOUNT_IS_WEIGHT" => amount.is_weight = s.boolean(f),
                "USE_FOR" => m.use_for = text(f),
                "NOTES" => m.notes = text(f),
                _ => return false,
            }
            true
        });
        m.amount = amount.resolve();
        m
    }

    fn water(&mut self, node: Node<'_, '_>, path: &str) -> Water {
        let mut w = Water::default();
        w.name = self.record(node, path, WATER_ELEMENTS, |s, f| {
            match f.name {
                "AMOUNT" => w.amount = s.number(f).map(Litres::new),
                "CALCIUM" => w.calcium = s.number(f).map(Ppm::new),
                "BICARBONATE" => w.bicarbonate = s.number(f).map(Ppm::new),
                "SULFATE" => w.sulfate = s.number(f).map(Ppm::new),
                "CHLORIDE" => w.chloride = s.number(f).map(Ppm::new),
                "SODIUM" => w.sodium = s.number(f).map(Ppm::new),
                "MAGNESIUM" => w.magnesium = s.number(f).map(Ppm::new),
                "PH" => w.ph = s.number(f),
                "NOTES" => w.notes = text(f),
                _ => return false,
            }
            true
        });
        w
    }

    /// Reads a boolean field: `TRUE` or `FALSE` in any case. An empty field
    /// is absent.
    fn boolean(&mut self, f: &Field<'_>) -> Option<bool> {
        let t = f.text.trim();
        if t.is_empty() {
            None
        } else if t.eq_ignore_ascii_case("true") {
            Some(true)
        } else if t.eq_ignore_ascii_case("false") {
            Some(false)
        } else {
            let text = f.text.to_owned();
            self.warn(f.path.clone(), WarningKind::UnreadableBoolean { text });
            None
        }
    }

    /// Reads an integer field: the whole text must be a whole,
    /// non-negative number that fits a `u32` (surrounding whitespace aside).
    /// Anything else is a single [`WarningKind::UnreadableNumber`].
    fn integer(&mut self, f: &Field<'_>) -> Option<u32> {
        match leading(f.text) {
            Leading::Empty => None,
            Leading::Number {
                value,
                trailing: false,
            } if value.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&value) => {
                // Checked above: whole and in range, so the cast is exact.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some(value as u32)
            }
            Leading::Number { .. } | Leading::Unreadable => {
                let text = f.text.to_owned();
                self.warn(f.path.clone(), WarningKind::UnreadableNumber { text });
                None
            }
        }
    }

    /// Reads a 1.0 display field (Appendix A of the 1.0 spec, e.g.
    /// `IBU`): like [`number`](Self::number), but in 1.0 the unit text
    /// writers put after the number (`32.4 IBU`) is expected, not reported.
    /// In 1.1 these fields are plain numbers, so it is reported as usual.
    fn display_number(&mut self, f: &Field<'_>) -> Option<f64> {
        self.leading_number(f, self.version == Version::V1_1)
    }

    /// Reads a numeric field by the leading-number rule.
    fn number(&mut self, f: &Field<'_>) -> Option<f64> {
        self.leading_number(f, true)
    }

    /// Reads the leading number of a field, reporting trailing text only
    /// when `report_trailing`.
    fn leading_number(&mut self, f: &Field<'_>, report_trailing: bool) -> Option<f64> {
        match leading(f.text) {
            Leading::Empty => None,
            Leading::Number { value, trailing } => {
                if trailing && report_trailing {
                    let text = f.text.to_owned();
                    self.warn(f.path.clone(), WarningKind::TrailingText { text });
                }
                Some(value)
            }
            Leading::Unreadable => {
                let text = f.text.to_owned();
                self.warn(f.path.clone(), WarningKind::UnreadableNumber { text });
                None
            }
        }
    }

    /// Reads an enumerated field; an empty field is absent.
    fn enumerated<E: Lenient>(&mut self, f: &Field<'_>) -> Option<E> {
        if f.text.trim().is_empty() {
            return None;
        }
        let value = E::parse(f.text);
        if value.is_other() {
            let text = f.text.to_owned();
            self.warn(f.path.clone(), WarningKind::UnknownValue { text });
        }
        Some(value)
    }
}

/// The `AMOUNT` and `AMOUNT_IS_WEIGHT` fields of a Misc or Yeast, collected
/// while walking the record since either may come first.
#[derive(Default)]
struct AmountFields {
    value: Option<f64>,
    is_weight: Option<bool>,
}

impl AmountFields {
    /// The amount, a volume unless flagged as a weight (the spec's default).
    fn resolve(self) -> Option<Amount> {
        let value = self.value?;
        Some(if self.is_weight == Some(true) {
            Amount::Weight(Kilograms::new(value))
        } else {
            Amount::Volume(Litres::new(value))
        })
    }
}

/// Reads a date field; an empty field is absent.
fn date(f: &Field<'_>) -> Option<RecordDate> {
    (!f.text.trim().is_empty()).then(|| RecordDate::parse(f.text))
}

/// Reads a text field, trimmed; an empty field is absent.
fn text(f: &Field<'_>) -> Option<String> {
    let text = f.text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}
