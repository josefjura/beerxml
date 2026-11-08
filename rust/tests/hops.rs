//! Hops, both in an ingredient library (`HOPS`) and inside a recipe.

use std::path::Path;

use beerxml::units::{Kilograms, Minutes, Percent};
use beerxml::{Hop, HopForm, HopType, HopUse, Warning, WarningKind};

fn sample(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../samples")
            .join(name),
    )
    .unwrap()
}

fn hops_doc(hop_fields: &str) -> String {
    format!("<HOPS><HOP><NAME>Saaz</NAME>{hop_fields}</HOP></HOPS>")
}

fn one_hop(hop_fields: &str) -> (Hop, Vec<Warning>) {
    let r = beerxml::read(hops_doc(hop_fields).as_bytes()).unwrap();
    let [hop] = <[Hop; 1]>::try_from(r.document.hops).unwrap();
    (hop, r.warnings)
}

#[test]
fn hop_library_reads_every_field() {
    let r = beerxml::read(&sample("original/hops.xml")).unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let names: Vec<_> = r.document.hops.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(names.first(), Some(&"Cascade"));
    assert_eq!(names.len(), 5);

    let cascade = &r.document.hops[0];
    assert_eq!(cascade.origin.as_deref(), Some("US"));
    assert_eq!(cascade.alpha, Some(Percent::new(5.5)));
    assert_eq!(cascade.amount, Some(Kilograms::new(0.0)));
    assert_eq!(cascade.usage, Some(HopUse::Boil));
    assert_eq!(cascade.time, Some(Minutes::new(0.0)));
    assert_eq!(cascade.kind, Some(HopType::Both));
    assert_eq!(cascade.form, Some(HopForm::Pellet));
    assert_eq!(cascade.beta, Some(Percent::new(6.0)));
    assert_eq!(cascade.hsi, Some(Percent::new(50.0)));
    assert!(
        cascade
            .notes
            .as_deref()
            .unwrap()
            .starts_with("Use For: American ales")
    );
}

#[test]
fn every_hop_field_is_read() {
    let (hop, warnings) = one_hop(
        "<VERSION>1</VERSION><ALPHA>3.5</ALPHA><AMOUNT>0.05</AMOUNT><USE>Aroma</USE>\
         <TIME>15</TIME><NOTES> Noble </NOTES><TYPE>Aroma</TYPE><FORM>Leaf</FORM>\
         <BETA>4.0</BETA><HSI>25</HSI><ORIGIN>Czech Republic</ORIGIN>\
         <SUBSTITUTES>Tettnang</SUBSTITUTES><HUMULENE>20</HUMULENE>\
         <CARYOPHYLLENE>7</CARYOPHYLLENE><COHUMULONE>25</COHUMULONE><MYRCENE>25</MYRCENE>",
    );
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(hop.name, "Saaz");
    assert_eq!(hop.alpha, Some(Percent::new(3.5)));
    assert_eq!(hop.amount, Some(Kilograms::new(0.05)));
    assert_eq!(hop.usage, Some(HopUse::Aroma));
    assert_eq!(hop.time, Some(Minutes::new(15.0)));
    assert_eq!(hop.notes.as_deref(), Some("Noble"));
    assert_eq!(hop.kind, Some(HopType::Aroma));
    assert_eq!(hop.form, Some(HopForm::Leaf));
    assert_eq!(hop.beta, Some(Percent::new(4.0)));
    assert_eq!(hop.hsi, Some(Percent::new(25.0)));
    assert_eq!(hop.origin.as_deref(), Some("Czech Republic"));
    assert_eq!(hop.substitutes.as_deref(), Some("Tettnang"));
    assert_eq!(hop.humulene, Some(Percent::new(20.0)));
    assert_eq!(hop.caryophyllene, Some(Percent::new(7.0)));
    assert_eq!(hop.cohumulone, Some(Percent::new(25.0)));
    assert_eq!(hop.myrcene, Some(Percent::new(25.0)));
}

#[test]
fn absent_hop_fields_are_none() {
    let (hop, warnings) = one_hop("<ORIGIN></ORIGIN><USE/>");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(hop.alpha, None);
    assert_eq!(hop.usage, None);
    assert_eq!(hop.origin, None);
}

#[test]
fn enums_match_case_and_whitespace_insensitively() {
    let (hop, warnings) = one_hop("<USE>Dry hop </USE><TYPE>BITTERING</TYPE><FORM> plug</FORM>");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(hop.usage, Some(HopUse::DryHop));
    assert_eq!(hop.kind, Some(HopType::Bittering));
    assert_eq!(hop.form, Some(HopForm::Plug));

    let (hop, _) = one_hop("<USE>DRY HOP</USE>");
    assert_eq!(hop.usage, Some(HopUse::DryHop));
}

#[test]
fn unknown_enum_values_are_kept_with_a_warning() {
    let (hop, warnings) = one_hop("<USE>Whirlpool</USE><FORM>Cryo</FORM>");
    assert_eq!(hop.usage, Some(HopUse::Other("Whirlpool".into())));
    assert_eq!(hop.form, Some(HopForm::Other("Cryo".into())));
    assert_eq!(
        warnings,
        [
            Warning {
                path: "HOPS/HOP[1]/USE".into(),
                kind: WarningKind::UnknownValue {
                    text: "Whirlpool".into()
                },
            },
            Warning {
                path: "HOPS/HOP[1]/FORM".into(),
                kind: WarningKind::UnknownValue {
                    text: "Cryo".into()
                },
            },
        ]
    );
}

#[test]
fn hop_numbers_follow_the_leading_number_rule() {
    let (hop, warnings) = one_hop("<ALPHA>5.5 %</ALPHA><AMOUNT>lots</AMOUNT>");
    assert_eq!(hop.alpha, Some(Percent::new(5.5)));
    assert_eq!(hop.amount, None);
    let paths: Vec<_> = warnings.iter().map(|w| w.path.as_str()).collect();
    assert_eq!(paths, ["HOPS/HOP[1]/ALPHA", "HOPS/HOP[1]/AMOUNT"]);
}

#[test]
fn unknown_hop_elements_are_reported_only_in_1_1() {
    let (_, warnings) = one_hop("<INVENTORY>1 oz</INVENTORY><DISPLAY_AMOUNT>1 oz</DISPLAY_AMOUNT>");
    assert!(warnings.is_empty(), "{warnings:?}");

    let xml = r#"<HOPS xmlns="http://beerxml.com/v1.1"><HOP><NAME>Saaz</NAME><INVENTORY>1 oz</INVENTORY><DISPLAY_AMOUNT>1 oz</DISPLAY_AMOUNT></HOP></HOPS>"#;
    let r = beerxml::read(xml.as_bytes()).unwrap();
    assert_eq!(
        r.warnings,
        [Warning {
            path: "HOPS/HOP[1]/INVENTORY".into(),
            kind: WarningKind::UnknownElement,
        }]
    );
}

#[test]
fn recipe_hops_come_back_in_document_order() {
    let r = beerxml::read(&sample("original/recipes.xml")).unwrap();
    let burton = &r.document.recipes[0];
    assert_eq!(burton.name, "Burton Ale");
    let additions: Vec<_> = burton
        .hops
        .iter()
        .map(|h| {
            (
                h.name.as_str(),
                h.amount.unwrap().get(),
                h.usage.clone().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        additions,
        [
            ("Goldings, East Kent", 0.028_35, HopUse::Boil),
            ("Northern Brewer", 0.014_175, HopUse::Boil),
            ("Fuggles", 0.014_175, HopUse::Boil),
            ("Fuggles", 0.021_262, HopUse::DryHop),
        ]
    );
    assert!(
        r.document.hops.is_empty(),
        "recipe hops are not top-level hops"
    );
}

#[test]
fn recipe_hop_warnings_carry_the_full_path() {
    let xml = "<RECIPES><RECIPE><NAME>A</NAME><HOPS>\
               <HOP><NAME>One</NAME></HOP><HOP><NAME>Two</NAME><USE>Hopback</USE></HOP>\
               </HOPS></RECIPE></RECIPES>";
    let r = beerxml::read(xml.as_bytes()).unwrap();
    let names: Vec<_> = r.document.recipes[0]
        .hops
        .iter()
        .map(|h| h.name.as_str())
        .collect();
    assert_eq!(names, ["One", "Two"]);
    assert_eq!(r.warnings[0].path, "RECIPES/RECIPE[1]/HOPS/HOP[2]/USE");
}
