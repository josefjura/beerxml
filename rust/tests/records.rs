//! Styles, Equipment, Mashes and Mash steps, both as top-level lists and
//! inside a recipe, and the complete Recipe record.

use std::path::Path;

use beerxml::units::{
    Celsius, Co2Volumes, Days, Ibu, Kilograms, Litres, Minutes, Percent, SpecificGravity, Srm,
};
use beerxml::{
    HopUse, IbuMethod, IsoDate, MashStepType, Read, RecipeType, RecordDate, StyleType, Warning,
    WarningKind,
};

fn sample(name: &str) -> Read {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../samples")
            .join(name),
    )
    .unwrap();
    beerxml::read(&bytes).unwrap()
}

fn read(xml: &str) -> Read {
    beerxml::read(xml.as_bytes()).unwrap()
}

fn warning(path: &str, kind: WarningKind) -> Warning {
    Warning {
        path: path.to_owned(),
        kind,
    }
}

fn one<T>(records: Vec<T>) -> T {
    let [record] = <[T; 1]>::try_from(records).unwrap_or_else(|v| panic!("{} records", v.len()));
    record
}

fn names<T>(records: &[T], name: impl Fn(&T) -> &str) -> Vec<&str> {
    records.iter().map(name).collect()
}

// --- Styles -----------------------------------------------------------------

#[test]
fn style_list_reads_as_in_the_file() {
    let r = sample("original/style.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(r.document.styles.len(), 5);

    let s = &r.document.styles[0];
    assert_eq!(s.name, "American Wheat");
    assert_eq!(s.category.as_deref(), Some("Light Ale"));
    assert_eq!(s.category_number.as_deref(), Some("3"));
    assert_eq!(s.style_letter.as_deref(), Some("B"));
    assert_eq!(s.style_guide.as_deref(), Some("BJCP 1999"));
    assert_eq!(s.kind, Some(StyleType::Mixed));
    assert_eq!(s.og_min, Some(SpecificGravity::new(1.035)));
    assert_eq!(s.og_max, Some(SpecificGravity::new(1.055)));
    assert_eq!(s.fg_min, Some(SpecificGravity::new(1.008)));
    assert_eq!(s.fg_max, Some(SpecificGravity::new(1.015)));
    assert_eq!(s.ibu_min, Some(Ibu::new(10.0)));
    assert_eq!(s.ibu_max, Some(Ibu::new(30.0)));
    assert_eq!(s.color_min, Some(Srm::new(2.0)));
    assert_eq!(s.color_max, Some(Srm::new(8.0)));
    assert_eq!(s.carb_min, Some(Co2Volumes::new(2.3)));
    assert_eq!(s.carb_max, Some(Co2Volumes::new(2.6)));
    assert_eq!(s.abv_min, Some(Percent::new(3.7)));
    assert_eq!(s.abv_max, Some(Percent::new(5.5)));
    assert!(s.notes.as_deref().unwrap().starts_with("Wheat beer brewed"));
    assert!(s.profile.as_deref().unwrap().starts_with("Light, crisp"));
    assert!(s.ingredients.as_deref().unwrap().starts_with("Pale malt."));
    assert!(s.examples.as_deref().unwrap().starts_with("Sam Adam's"));
}

#[test]
fn every_style_type_is_recognised() {
    for (text, kind) in [
        ("lager", StyleType::Lager),
        ("Ale", StyleType::Ale),
        ("MEAD", StyleType::Mead),
        ("Wheat", StyleType::Wheat),
        ("Mixed", StyleType::Mixed),
        ("cider", StyleType::Cider),
    ] {
        let xml = format!("<STYLES><STYLE><NAME>S</NAME><TYPE>{text}</TYPE></STYLE></STYLES>");
        assert_eq!(one(read(&xml).document.styles).kind, Some(kind), "{text}");
    }
}

#[test]
fn unknown_style_type_is_kept_and_reported() {
    let r = read("<STYLES><STYLE><NAME>S</NAME><TYPE>Kombucha</TYPE></STYLE></STYLES>");
    assert_eq!(
        one(r.document.styles).kind,
        Some(StyleType::Other("Kombucha".into()))
    );
    assert_eq!(
        r.warnings,
        [warning(
            "STYLES/STYLE[1]/TYPE",
            WarningKind::UnknownValue {
                text: "Kombucha".into()
            }
        )]
    );
}

// --- Equipment --------------------------------------------------------------

#[test]
fn equipment_list_reads_as_in_the_file() {
    let r = sample("original/equipment.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(
        names(&r.document.equipments, |e| &e.name),
        [
            "Brew Pot (4 Gallon)",
            "Brew Pot (6+gal) and Igloo/Gott Cooler (5 Gal)"
        ]
    );

    let e = &r.document.equipments[0];
    assert_eq!(e.boil_size, Some(Litres::new(12.306_126)));
    assert_eq!(e.batch_size, Some(Litres::new(18.927_168)));
    assert_eq!(e.tun_volume, Some(Litres::new(15.141_734)));
    assert_eq!(e.tun_weight, Some(Kilograms::new(1.814_368)));
    assert_eq!(e.tun_specific_heat, Some(0.12));
    assert_eq!(e.top_up_water, Some(Litres::new(8.517_226)));
    assert_eq!(e.trub_chiller_loss, Some(Litres::new(0.0)));
    assert_eq!(e.evap_rate, Some(Percent::new(9.0)));
    assert_eq!(e.boil_time, Some(Minutes::new(60.0)));
    assert_eq!(e.calc_boil_volume, Some(true));
    assert_eq!(e.lauter_deadspace, Some(Litres::new(0.946_358)));
    assert_eq!(e.top_up_kettle, Some(Litres::new(0.0)));
    assert_eq!(e.hop_utilization, Some(Percent::new(100.0)));
    assert!(e.notes.as_deref().unwrap().starts_with("Simple Brew Pot"));
}

// --- Mashes and Mash steps --------------------------------------------------

#[test]
fn mash_list_reads_as_in_the_file_with_steps_in_order() {
    let r = sample("original/mash.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(r.document.mashes.len(), 5);

    let m = &r.document.mashes[0];
    assert_eq!(m.name, "Temperature Mash, 1 Step, Light Body");
    assert_eq!(m.grain_temp, Some(Celsius::new(22.222_222)));
    assert_eq!(m.tun_temp, Some(Celsius::new(22.222_222)));
    assert_eq!(m.sparge_temp, Some(Celsius::new(75.555_556)));
    assert_eq!(m.ph, Some(5.4));
    assert_eq!(m.tun_weight, Some(Kilograms::new(0.0)));
    assert_eq!(m.tun_specific_heat, Some(0.12));
    assert_eq!(m.equip_adjust, Some(false));
    assert!(m.notes.as_deref().unwrap().starts_with("Temperature mash"));

    assert_eq!(
        names(&m.mash_steps, |s| &s.name),
        ["Saccharification", "Mash Out"]
    );
    let s = &m.mash_steps[0];
    assert_eq!(s.kind, Some(MashStepType::Infusion));
    assert_eq!(s.infuse_amount, Some(Litres::new(11.829_48)));
    assert_eq!(s.step_time, Some(Minutes::new(75.0)));
    assert_eq!(s.step_temp, Some(Celsius::new(65.555_556)));
    assert_eq!(s.ramp_time, Some(Minutes::new(15.0)));
    assert_eq!(s.end_temp, Some(Celsius::new(65.555_556)));
    assert_eq!(
        s.description.as_deref(),
        Some("Add 12.50 qt of water at 161.4 F")
    );
    // Display text with units, as the spec and BeerSmith write them.
    assert_eq!(s.water_grain_ratio.as_deref(), Some("1.25"));
    assert_eq!(s.decoction_amt.as_deref(), Some("0.00 qt"));
    assert_eq!(s.infuse_temp.as_deref(), Some("161.4 F"));
    assert_eq!(m.mash_steps[1].kind, Some(MashStepType::Temperature));
}

#[test]
fn mash_step_list_reads_as_a_top_level_list() {
    let r = read(
        "<MASH_STEPS>\
           <MASH_STEP><NAME>Protein Rest</NAME><TYPE>Decoction</TYPE>\
             <STEP_TEMP>52</STEP_TEMP><STEP_TIME>15</STEP_TIME></MASH_STEP>\
           <MASH_STEP><NAME>Sacch</NAME><TYPE>Step</TYPE></MASH_STEP>\
         </MASH_STEPS>",
    );
    let steps = &r.document.mash_steps;
    assert_eq!(names(steps, |s| &s.name), ["Protein Rest", "Sacch"]);
    assert_eq!(steps[0].kind, Some(MashStepType::Decoction));
    assert_eq!(steps[0].step_temp, Some(Celsius::new(52.0)));
    assert_eq!(steps[0].step_time, Some(Minutes::new(15.0)));
    assert_eq!(steps[1].kind, Some(MashStepType::Other("Step".into())));
    assert_eq!(
        r.warnings,
        [warning(
            "MASH_STEPS/MASH_STEP[2]/TYPE",
            WarningKind::UnknownValue {
                text: "Step".into()
            }
        )]
    );
}

#[test]
fn mash_step_warnings_carry_their_index() {
    let r = read(
        "<MASHS><MASH><NAME>M</NAME><MASH_STEPS>\
           <MASH_STEP><NAME>A</NAME></MASH_STEP>\
           <MASH_STEP><NAME>B</NAME><STEP_TEMP>hot</STEP_TEMP></MASH_STEP>\
         </MASH_STEPS></MASH></MASHS>",
    );
    assert_eq!(
        r.warnings,
        [warning(
            "MASHS/MASH[1]/MASH_STEPS/MASH_STEP[2]/STEP_TEMP",
            WarningKind::UnreadableNumber { text: "hot".into() }
        )]
    );
}

// --- Recipe -----------------------------------------------------------------

#[test]
fn recipe_reads_every_field_as_in_the_file() {
    let r = sample("original/recipes.xml");
    let burton = &r.document.recipes[0];
    assert_eq!(burton.name, "Burton Ale");
    assert_eq!(burton.kind, Some(RecipeType::AllGrain));
    assert_eq!(burton.brewer.as_deref(), Some("Brad Smith"));
    assert_eq!(burton.asst_brewer, None, "<ASST_BREWER /> is empty");
    assert_eq!(burton.notes, None, "<NOTES /> is empty");
    assert!(
        burton
            .taste_notes
            .as_deref()
            .unwrap()
            .starts_with("A smooth tasting pale ale")
    );
    assert_eq!(burton.taste_rating, Some(38.0));
    assert_eq!(burton.carbonation, Some(Co2Volumes::new(2.4)));
    assert_eq!(burton.fermentation_stages, Some(2));
    assert_eq!(burton.primary_age, Some(Days::new(4.0)));
    assert_eq!(burton.primary_temp, Some(Celsius::new(20.0)));
    assert_eq!(burton.secondary_age, Some(Days::new(7.0)));
    assert_eq!(burton.secondary_temp, Some(Celsius::new(20.0)));
    assert_eq!(burton.tertiary_age, Some(Days::new(0.0)));
    assert_eq!(burton.tertiary_temp, None);
    assert_eq!(burton.age, Some(Days::new(14.0)));
    assert_eq!(burton.age_temp, Some(Celsius::new(11.111)));
    assert_eq!(burton.carbonation_used.as_deref(), Some("12 PSI"));
    assert_eq!(
        burton.date,
        Some(RecordDate {
            text: "4/6/2003".into(),
            iso: None
        })
    );
    assert_eq!(burton.est_og, Some(SpecificGravity::new(1.056)));
    assert_eq!(burton.est_fg, Some(SpecificGravity::new(1.015)));
    assert_eq!(burton.est_color, Some(Srm::new(7.0)));
    assert_eq!(burton.ibu, Some(Ibu::new(32.4)));
    assert_eq!(burton.ibu_method, Some(IbuMethod::Tinseth));
    assert_eq!(burton.est_abv, Some(Percent::new(5.3)));
    assert_eq!(burton.abv, Some(Percent::new(5.1)));
    assert_eq!(burton.actual_efficiency, Some(Percent::new(69.7)));
    assert_eq!(burton.calories, Some(242.0));

    let style = burton.style.as_ref().expect("Burton Ale has a style");
    assert_eq!(style.name, "English Pale Ale");
    assert_eq!(style.kind, Some(StyleType::Ale));
    let equipment = burton.equipment.as_ref().expect("Burton Ale has equipment");
    assert_eq!(
        equipment.name,
        "Brew Pot (6+gal) and Igloo/Gott Cooler (5 Gal)"
    );
    let mash = burton.mash.as_ref().expect("Burton Ale has a mash");
    assert_eq!(mash.name, "Single Infusion, Full Body");
    assert_eq!(
        names(&mash.mash_steps, |s| &s.name),
        ["Mash In", "Mash Out"]
    );
}

#[test]
fn recipe_carbonation_and_priming_fields() {
    let r = read(
        "<RECIPES><RECIPE><NAME>R</NAME>\
           <FORCED_CARBONATION>TRUE</FORCED_CARBONATION>\
           <PRIMING_SUGAR_NAME> Corn Sugar </PRIMING_SUGAR_NAME>\
           <CARBONATION_TEMP>4.5</CARBONATION_TEMP>\
           <PRIMING_SUGAR_EQUIV>1.0</PRIMING_SUGAR_EQUIV>\
           <KEG_PRIMING_FACTOR>0.5</KEG_PRIMING_FACTOR>\
           <RATING>4</RATING>\
           <IBU_METHOD>Garetz</IBU_METHOD>\
         </RECIPE></RECIPES>",
    );
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let recipe = one(r.document.recipes);
    assert_eq!(recipe.forced_carbonation, Some(true));
    assert_eq!(recipe.priming_sugar_name.as_deref(), Some("Corn Sugar"));
    assert_eq!(recipe.carbonation_temp, Some(Celsius::new(4.5)));
    assert_eq!(recipe.priming_sugar_equiv, Some(1.0));
    assert_eq!(recipe.keg_priming_factor, Some(0.5));
    assert_eq!(recipe.rating, Some(4.0));
    assert_eq!(recipe.ibu_method, Some(IbuMethod::Garetz));
}

#[test]
fn ibus_spelling_reads_into_ibu() {
    let r = read("<RECIPES><RECIPE><NAME>R</NAME><IBUS>21</IBUS></RECIPE></RECIPES>");
    assert_eq!(one(r.document.recipes).ibu, Some(Ibu::new(21.0)));
}

#[test]
fn unknown_recipe_type_and_ibu_method_are_kept_and_reported() {
    let r = read(
        "<RECIPES><RECIPE><NAME>R</NAME><TYPE>BIAB</TYPE>\
           <IBU_METHOD>Daniels</IBU_METHOD></RECIPE></RECIPES>",
    );
    let recipe = one(r.document.recipes);
    assert_eq!(recipe.kind, Some(RecipeType::Other("BIAB".into())));
    assert_eq!(recipe.ibu_method, Some(IbuMethod::Other("Daniels".into())));
    assert_eq!(
        r.warnings,
        [
            warning(
                "RECIPES/RECIPE[1]/TYPE",
                WarningKind::UnknownValue {
                    text: "BIAB".into()
                }
            ),
            warning(
                "RECIPES/RECIPE[1]/IBU_METHOD",
                WarningKind::UnknownValue {
                    text: "Daniels".into()
                }
            ),
        ]
    );
}

#[test]
fn every_recipe_type_is_recognised() {
    for (text, kind) in [
        ("Extract", RecipeType::Extract),
        ("partial  mash", RecipeType::PartialMash),
        ("ALL GRAIN", RecipeType::AllGrain),
    ] {
        let xml = format!("<RECIPES><RECIPE><NAME>R</NAME><TYPE>{text}</TYPE></RECIPE></RECIPES>");
        assert_eq!(one(read(&xml).document.recipes).kind, Some(kind), "{text}");
    }
}

#[test]
fn recipe_without_optional_sections_has_none() {
    let r = read("<RECIPES><RECIPE><NAME>Bare</NAME></RECIPE></RECIPES>");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let recipe = one(r.document.recipes);
    assert_eq!(recipe.style, None);
    assert_eq!(recipe.equipment, None);
    assert_eq!(recipe.mash, None);
    assert_eq!(recipe.date, None);
}

#[test]
fn iso_recipe_date_is_parsed() {
    let r = sample("v1.1_sample.xml");
    assert_eq!(
        r.document.recipes[0].date,
        Some(RecordDate {
            text: "2024-01-13".into(),
            iso: Some(IsoDate {
                year: 2024,
                month: 1,
                day: 13
            })
        })
    );
}

#[test]
fn nested_record_warnings_carry_their_path() {
    let r = read(
        "<RECIPES><RECIPE><NAME>R</NAME>\
           <STYLE><NAME>S</NAME><OG_MIN>low</OG_MIN></STYLE>\
           <EQUIPMENT><NAME>E</NAME><CALC_BOIL_VOLUME>yes</CALC_BOIL_VOLUME></EQUIPMENT>\
         </RECIPE></RECIPES>",
    );
    assert_eq!(
        r.warnings,
        [
            warning(
                "RECIPES/RECIPE[1]/STYLE/OG_MIN",
                WarningKind::UnreadableNumber { text: "low".into() }
            ),
            warning(
                "RECIPES/RECIPE[1]/EQUIPMENT/CALC_BOIL_VOLUME",
                WarningKind::UnreadableBoolean { text: "yes".into() }
            ),
        ]
    );
}

// --- Real-world sample (josefjura/asgard#126) --------------------------------

#[test]
fn realworld_sample_reads_everything_asgard_needs() {
    let r = sample("brewtlery_unetice_realworld_sample.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let recipe = one(r.document.recipes);

    assert_eq!(recipe.name, "Únětický Vzor 10.7° (G70 30L)");
    assert_eq!(recipe.brewer.as_deref(), Some("Domácí Sládek"));
    assert_eq!(recipe.kind, Some(RecipeType::AllGrain));
    // Padded numbers read exactly.
    assert_eq!(recipe.batch_size, Some(Litres::new(30.0)));
    assert_eq!(recipe.boil_size, Some(Litres::new(36.0)));
    assert_eq!(recipe.boil_time, Some(Minutes::new(90.0)));
    assert_eq!(recipe.efficiency, Some(Percent::new(75.0)));
    assert_eq!(recipe.og, Some(SpecificGravity::new(1.043)));
    assert_eq!(recipe.fg, Some(SpecificGravity::new(1.010)));

    let style = recipe.style.as_ref().expect("style");
    assert_eq!(style.name, "Czech Premium Pale Lager");
    assert_eq!(style.category_number.as_deref(), Some("3B"));
    assert_eq!(style.kind, Some(StyleType::Lager));
    assert_eq!(style.og_min, Some(SpecificGravity::new(1.044)));
    assert_eq!(style.og_max, Some(SpecificGravity::new(1.060)));
    assert_eq!(style.fg_min, Some(SpecificGravity::new(1.013)));
    assert_eq!(style.fg_max, Some(SpecificGravity::new(1.017)));
    assert_eq!(style.ibu_min, Some(Ibu::new(30.0)));
    assert_eq!(style.ibu_max, Some(Ibu::new(45.0)));
    assert_eq!(style.color_min, Some(Srm::new(6.896)));
    assert_eq!(style.color_max, Some(Srm::new(11.82)));

    let hops: Vec<_> = recipe
        .hops
        .iter()
        .map(|h| (h.name.as_str(), h.amount, h.usage.clone(), h.time))
        .collect();
    assert_eq!(
        hops,
        [
            (
                "Sladek",
                Some(Kilograms::new(0.03)),
                Some(HopUse::Boil),
                Some(Minutes::new(60.0))
            ),
            (
                "Saaz",
                Some(Kilograms::new(0.04)),
                Some(HopUse::Boil),
                Some(Minutes::new(20.0))
            ),
            (
                "Saaz",
                Some(Kilograms::new(0.05)),
                Some(HopUse::Boil),
                Some(Minutes::new(5.0))
            ),
            (
                "Saaz",
                Some(Kilograms::new(0.03)),
                Some(HopUse::Aroma),
                Some(Minutes::new(15.0))
            ),
        ]
    );
    assert_eq!(recipe.hops[3].notes.as_deref(), Some("Whirlpool při 80°C"));

    assert_eq!(
        names(&recipe.fermentables, |f| &f.name),
        ["Pilsner (Weyermann)", "Carapils (Weyermann)"]
    );
    assert_eq!(names(&recipe.yeasts, |y| &y.name), ["NovaLager"]);

    let mash = recipe.mash.as_ref().expect("mash");
    assert_eq!(mash.name, "Dvojitá dekokce (Simulace krokem)");
    assert_eq!(mash.grain_temp, Some(Celsius::new(20.0)));
    let steps: Vec<_> = mash
        .mash_steps
        .iter()
        .map(|s| (s.name.as_str(), s.step_temp, s.step_time))
        .collect();
    assert_eq!(
        steps,
        [
            (
                "Vystírka (Protein Rest)",
                Some(Celsius::new(52.0)),
                Some(Minutes::new(5.0))
            ),
            (
                "Nižší cukrotvorná (Maltose)",
                Some(Celsius::new(63.0)),
                Some(Minutes::new(40.0))
            ),
            (
                "Vyšší cukrotvorná (Dextrin)",
                Some(Celsius::new(72.0)),
                Some(Minutes::new(25.0))
            ),
            (
                "Odrmutování (Mash Out)",
                Some(Celsius::new(78.0)),
                Some(Minutes::new(10.0))
            ),
        ]
    );
    assert_eq!(recipe.equipment, None);
}
