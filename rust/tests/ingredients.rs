//! Fermentables, Yeasts, Miscs and Waters, both as ingredient libraries and
//! inside a recipe.

use std::path::Path;

use beerxml::units::SpecificGravity;
use beerxml::units::{Amount, Celsius, Kilograms, Lintner, Litres, Minutes, Percent, Ppm, Srm};
use beerxml::{
    Fermentable, FermentableType, IsoDate, Misc, MiscType, MiscUse, Read, RecordDate, Warning,
    WarningKind, Water, Yeast, YeastFlocculation, YeastForm, YeastType,
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

// --- Fermentables -----------------------------------------------------------

#[test]
fn fermentable_library_reads_as_in_the_file() {
    let r = sample("original/grain.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let all = &r.document.fermentables;
    assert_eq!(all.len(), 4);
    assert_eq!(all[0].name, "Acid Malt");

    let acid = &all[0];
    assert_eq!(acid.kind, Some(FermentableType::Grain));
    assert_eq!(acid.amount, Some(Kilograms::new(0.0)));
    assert_eq!(acid.yield_, Some(Percent::new(58.7)));
    assert_eq!(acid.color, Some(Srm::new(3.0)));
    assert_eq!(acid.add_after_boil, Some(false));
    assert_eq!(acid.origin.as_deref(), Some("Germany"));
    assert_eq!(acid.supplier, None);
    assert!(
        acid.notes
            .as_deref()
            .unwrap()
            .starts_with("Acid malt contains")
    );
    assert_eq!(acid.coarse_fine_diff, Some(Percent::new(1.5)));
    assert_eq!(acid.moisture, Some(Percent::new(4.0)));
    assert_eq!(acid.diastatic_power, Some(Lintner::new(0.0)));
    assert_eq!(acid.protein, Some(Percent::new(6.0)));
    assert_eq!(acid.max_in_batch, Some(Percent::new(10.0)));
    assert_eq!(acid.recommend_mash, Some(true));
    assert_eq!(acid.ibu_gal_per_lb, Some(0.0));
    assert_eq!(acid.potential, Some(SpecificGravity::new(1.027)));
}

#[test]
fn every_fermentable_type_matches_case_insensitively() {
    for (text, kind) in [
        ("grain", FermentableType::Grain),
        ("SUGAR", FermentableType::Sugar),
        ("Extract", FermentableType::Extract),
        ("dry  extract", FermentableType::DryExtract),
        ("Adjunct ", FermentableType::Adjunct),
    ] {
        let r = read(&format!(
            "<FERMENTABLES><FERMENTABLE><NAME>X</NAME><TYPE>{text}</TYPE></FERMENTABLE></FERMENTABLES>"
        ));
        assert_eq!(one(r.document.fermentables).kind, Some(kind), "{text:?}");
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }
}

#[test]
fn diastatic_power_typo_in_the_schema_is_read_too() {
    let r = read(
        "<FERMENTABLES><FERMENTABLE><NAME>X</NAME>\
         <DISASTATIC_POWER>120</DISASTATIC_POWER></FERMENTABLE></FERMENTABLES>",
    );
    assert_eq!(
        one(r.document.fermentables).diastatic_power,
        Some(Lintner::new(120.0))
    );
}

// --- Yeasts -----------------------------------------------------------------

#[test]
fn yeast_library_reads_as_in_the_file() {
    let r = sample("original/yeast.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let all = &r.document.yeasts;
    assert_eq!(all.len(), 5);

    let ale = &all[0];
    assert_eq!(ale.name, "English Ale");
    assert_eq!(ale.kind, Some(YeastType::Ale));
    assert_eq!(ale.form, Some(YeastForm::Liquid));
    assert_eq!(ale.amount, Some(Amount::Volume(Litres::new(0.035))));
    assert_eq!(ale.laboratory.as_deref(), Some("White Labs"));
    assert_eq!(ale.product_id.as_deref(), Some("WLP002"));
    assert_eq!(ale.min_temperature, Some(Celsius::new(18.3)));
    assert_eq!(ale.max_temperature, Some(Celsius::new(20.0)));
    assert_eq!(ale.flocculation, Some(YeastFlocculation::VeryHigh));
    assert_eq!(ale.attenuation, Some(Percent::new(66.5)));
    assert!(
        ale.notes
            .as_deref()
            .unwrap()
            .starts_with("Classic ESB strain")
    );
    assert!(
        ale.best_for
            .as_deref()
            .unwrap()
            .starts_with("English Pale Ale")
    );
    assert_eq!(ale.max_reuse, Some(5));
    assert_eq!(ale.times_cultured, Some(0));
    assert_eq!(ale.add_to_secondary, Some(false));
    assert_eq!(
        ale.culture_date,
        Some(RecordDate {
            text: "8/31/2003".into(),
            iso: None
        })
    );
}

#[test]
fn yeast_enums_match_case_insensitively() {
    let r = read(
        "<YEASTS><YEAST><NAME>X</NAME><TYPE>CHAMPAGNE</TYPE><FORM>slant</FORM>\
         <FLOCCULATION>very high</FLOCCULATION></YEAST></YEASTS>",
    );
    let y = one(r.document.yeasts);
    assert_eq!(y.kind, Some(YeastType::Champagne));
    assert_eq!(y.form, Some(YeastForm::Slant));
    assert_eq!(y.flocculation, Some(YeastFlocculation::VeryHigh));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn yeast_amount_is_weight_when_flagged() {
    let r = read(
        "<YEASTS><YEAST><NAME>X</NAME><AMOUNT>0.011</AMOUNT>\
         <AMOUNT_IS_WEIGHT>True</AMOUNT_IS_WEIGHT></YEAST></YEASTS>",
    );
    assert_eq!(
        one(r.document.yeasts).amount,
        Some(Amount::Weight(Kilograms::new(0.011)))
    );
}

#[test]
fn iso_culture_date_is_parsed() {
    let r = read(
        "<YEASTS><YEAST><NAME>X</NAME><CULTURE_DATE> 2003-07-06 </CULTURE_DATE></YEAST></YEASTS>",
    );
    assert_eq!(
        one(r.document.yeasts).culture_date,
        Some(RecordDate {
            text: "2003-07-06".into(),
            iso: Some(IsoDate {
                year: 2003,
                month: 7,
                day: 6
            })
        })
    );
}

#[test]
fn integer_fields_with_a_fraction_are_unreadable() {
    let r = read("<YEASTS><YEAST><NAME>X</NAME><MAX_REUSE>2.5</MAX_REUSE></YEAST></YEASTS>");
    assert_eq!(one(r.document.yeasts).max_reuse, None);
    assert_eq!(
        r.warnings,
        [warning(
            "YEASTS/YEAST[1]/MAX_REUSE",
            WarningKind::UnreadableNumber { text: "2.5".into() }
        )]
    );
}

// --- Miscs ------------------------------------------------------------------

#[test]
fn misc_library_reads_as_in_the_file() {
    let r = sample("original/misc.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(
        names(&r.document.miscs, |m| &m.name),
        [
            "Apricot Extract",
            "Calcium Chloride",
            "Ginger Root",
            "Irish Moss",
            "Orange Peel, Bitter"
        ]
    );
    let apricot = &r.document.miscs[0];
    assert_eq!(apricot.kind, Some(MiscType::Flavor));
    assert_eq!(apricot.usage, Some(MiscUse::Bottling));
    assert_eq!(apricot.amount, Some(Amount::Volume(Litres::new(0.192_229))));
    assert_eq!(apricot.time, Some(Minutes::new(5.0)));
    assert_eq!(apricot.use_for.as_deref(), Some("Fruit Beer"));
    assert!(
        apricot
            .notes
            .as_deref()
            .unwrap()
            .starts_with("Apricot flavor")
    );
}

#[test]
fn misc_type_other_is_a_spec_value_not_an_unknown_one() {
    let r = read("<MISCS><MISC><NAME>X</NAME><TYPE>other</TYPE></MISC></MISCS>");
    let kind = one(r.document.miscs).kind.unwrap();
    assert_eq!(kind, MiscType::OtherType);
    assert_eq!(kind.as_str(), "Other");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn misc_enums_match_case_insensitively() {
    let r = read(
        "<MISCS><MISC><NAME>X</NAME><TYPE>WATER AGENT</TYPE><USE>secondary</USE></MISC></MISCS>",
    );
    let m = one(r.document.miscs);
    assert_eq!(m.kind, Some(MiscType::WaterAgent));
    assert_eq!(m.usage, Some(MiscUse::Secondary));
}

#[test]
fn misc_amount_flag_may_come_before_or_after_the_amount() {
    let r = read(
        "<MISCS><MISC><NAME>A</NAME><AMOUNT_IS_WEIGHT>TRUE</AMOUNT_IS_WEIGHT><AMOUNT>1</AMOUNT></MISC>\
         <MISC><NAME>B</NAME><AMOUNT>2</AMOUNT><AMOUNT_IS_WEIGHT>true</AMOUNT_IS_WEIGHT></MISC></MISCS>",
    );
    let amounts: Vec<_> = r.document.miscs.iter().map(|m| m.amount).collect();
    assert_eq!(
        amounts,
        [
            Some(Amount::Weight(Kilograms::new(1.0))),
            Some(Amount::Weight(Kilograms::new(2.0)))
        ]
    );
}

#[test]
fn misc_amount_without_flag_is_a_volume_as_the_spec_defaults() {
    let r = read("<MISCS><MISC><NAME>X</NAME><AMOUNT>0.5</AMOUNT></MISC></MISCS>");
    assert_eq!(
        one(r.document.miscs).amount,
        Some(Amount::Volume(Litres::new(0.5)))
    );
}

// --- Waters -----------------------------------------------------------------

#[test]
fn water_library_reads_as_in_the_file() {
    let r = sample("original/water.xml");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(r.document.waters.len(), 5);
    let burton = &r.document.waters[0];
    assert_eq!(burton.name, "Burton On Trent, UK");
    assert_eq!(burton.amount, Some(Litres::new(18.927_168)));
    assert_eq!(burton.calcium, Some(Ppm::new(295.0)));
    assert_eq!(burton.bicarbonate, Some(Ppm::new(300.0)));
    assert_eq!(burton.sulfate, Some(Ppm::new(725.0)));
    assert_eq!(burton.chloride, Some(Ppm::new(25.0)));
    assert_eq!(burton.sodium, Some(Ppm::new(55.0)));
    assert_eq!(burton.magnesium, Some(Ppm::new(45.0)));
    assert_eq!(burton.ph, Some(8.0));
    assert!(
        burton
            .notes
            .as_deref()
            .unwrap()
            .starts_with("Distinctive pale ales")
    );
}

// --- Inside recipes ---------------------------------------------------------

#[test]
fn recipes_expose_their_ingredients_in_document_order() {
    let r = read(
        "<RECIPES><RECIPE><NAME>R</NAME>\
         <FERMENTABLES><FERMENTABLE><NAME>F1</NAME></FERMENTABLE><FERMENTABLE><NAME>F2</NAME></FERMENTABLE></FERMENTABLES>\
         <YEASTS><YEAST><NAME>Y1</NAME></YEAST><YEAST><NAME>Y2</NAME></YEAST></YEASTS>\
         <MISCS><MISC><NAME>M1</NAME></MISC><MISC><NAME>M2</NAME></MISC></MISCS>\
         <WATERS><WATER><NAME>W1</NAME></WATER><WATER><NAME>W2</NAME></WATER></WATERS>\
         </RECIPE></RECIPES>",
    );
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let recipe = one(r.document.recipes);
    assert_eq!(
        names(&recipe.fermentables, |f: &Fermentable| &f.name),
        ["F1", "F2"]
    );
    assert_eq!(names(&recipe.yeasts, |y: &Yeast| &y.name), ["Y1", "Y2"]);
    assert_eq!(names(&recipe.miscs, |m: &Misc| &m.name), ["M1", "M2"]);
    assert_eq!(names(&recipe.waters, |w: &Water| &w.name), ["W1", "W2"]);
}

#[test]
fn recipes_in_the_samples_carry_their_ingredients() {
    let r = sample("original/recipes.xml");
    let burton = &r.document.recipes[0];
    assert_eq!(burton.name, "Burton Ale");
    assert!(!burton.fermentables.is_empty(), "no fermentables");
    assert!(!burton.yeasts.is_empty(), "no yeasts");

    let r = sample("v1.1_sample.xml");
    let recipe = &r.document.recipes[0];
    assert_eq!(recipe.fermentables[0].name, "Pilsner (Weyermann)");
    assert_eq!(recipe.fermentables[0].recommend_mash, Some(true));
    assert_eq!(recipe.miscs[0].name, "Irish Moss");
    assert_eq!(
        recipe.miscs[0].amount,
        Some(Amount::Weight(Kilograms::new(0.005)))
    );
    assert_eq!(recipe.yeasts[0].name, "NovaLager");
}

// --- Lenient rules ----------------------------------------------------------

#[test]
fn unknown_enum_values_fall_back_to_other_with_a_warning() {
    let r = read(
        "<YEASTS><YEAST><NAME>X</NAME><TYPE>Kveik</TYPE><FORM>Frozen</FORM>\
         <FLOCCULATION>Medium-High</FLOCCULATION></YEAST></YEASTS>",
    );
    let y = one(r.document.yeasts);
    assert_eq!(y.kind, Some(YeastType::Other("Kveik".into())));
    assert_eq!(y.form, Some(YeastForm::Other("Frozen".into())));
    assert_eq!(
        y.flocculation,
        Some(YeastFlocculation::Other("Medium-High".into()))
    );
    let paths: Vec<_> = r.warnings.iter().map(|w| w.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "YEASTS/YEAST[1]/TYPE",
            "YEASTS/YEAST[1]/FORM",
            "YEASTS/YEAST[1]/FLOCCULATION"
        ]
    );

    let r = read(
        "<FERMENTABLES><FERMENTABLE><NAME>X</NAME><TYPE>Fruit</TYPE></FERMENTABLE></FERMENTABLES>",
    );
    assert_eq!(
        one(r.document.fermentables).kind,
        Some(FermentableType::Other("Fruit".into()))
    );
    assert_eq!(
        r.warnings,
        [warning(
            "FERMENTABLES/FERMENTABLE[1]/TYPE",
            WarningKind::UnknownValue {
                text: "Fruit".into()
            }
        )]
    );

    let r = read("<MISCS><MISC><NAME>X</NAME><TYPE>Wood</TYPE><USE>Whirlpool</USE></MISC></MISCS>");
    let m = one(r.document.miscs);
    assert_eq!(m.kind, Some(MiscType::Other("Wood".into())));
    assert_eq!(m.usage, Some(MiscUse::Other("Whirlpool".into())));
    assert_eq!(r.warnings.len(), 2);
}

#[test]
fn booleans_read_in_any_case() {
    for (text, value) in [
        ("TRUE", true),
        ("true", true),
        (" True ", true),
        ("FALSE", false),
        ("false", false),
        ("fAlSe", false),
    ] {
        let r = read(&format!(
            "<FERMENTABLES><FERMENTABLE><NAME>X</NAME><ADD_AFTER_BOIL>{text}</ADD_AFTER_BOIL>\
             </FERMENTABLE></FERMENTABLES>"
        ));
        assert_eq!(
            one(r.document.fermentables).add_after_boil,
            Some(value),
            "{text:?}"
        );
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }
}

#[test]
fn unreadable_boolean_is_missing_with_a_warning() {
    let r = read(
        "<FERMENTABLES><FERMENTABLE><NAME>X</NAME><RECOMMEND_MASH>yes</RECOMMEND_MASH>\
         <ADD_AFTER_BOIL></ADD_AFTER_BOIL></FERMENTABLE></FERMENTABLES>",
    );
    let f = one(r.document.fermentables);
    assert_eq!(f.recommend_mash, None);
    assert_eq!(f.add_after_boil, None);
    assert_eq!(
        r.warnings,
        [warning(
            "FERMENTABLES/FERMENTABLE[1]/RECOMMEND_MASH",
            WarningKind::UnreadableBoolean { text: "yes".into() }
        )]
    );
}

#[test]
fn every_field_of_each_record_is_known_in_1_1() {
    // No unknown-element warnings for fields from the 1.1 schema.
    let ns = r#" xmlns="http://beerxml.com/v1.1" version="1.1""#;
    let r = read(&format!(
        "<RECIPES{ns}><RECIPE><NAME>R</NAME>\
         <FERMENTABLES><FERMENTABLE><NAME>F</NAME><VERSION>1.1</VERSION></FERMENTABLE></FERMENTABLES>\
         <YEASTS><YEAST><NAME>Y</NAME><VERSION>1.1</VERSION></YEAST></YEASTS>\
         <MISCS><MISC><NAME>M</NAME><VERSION>1.1</VERSION></MISC></MISCS>\
         <WATERS><WATER><NAME>W</NAME><VERSION>1.1</VERSION><BOGUS>1</BOGUS></WATER></WATERS>\
         </RECIPE></RECIPES>"
    ));
    assert_eq!(
        r.warnings,
        [warning(
            "RECIPES/RECIPE[1]/WATERS/WATER[1]/BOGUS",
            WarningKind::UnknownElement
        )]
    );
}
