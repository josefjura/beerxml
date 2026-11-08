//! The lenient reading rules, each checked on a small inline document.

use beerxml::{Read, ReadError, Version, Warning, WarningKind};

const V1_1_NS: &str = r#" xmlns="http://beerxml.com/v1.1""#;

fn read(xml: &str) -> Read {
    beerxml::read(xml.as_bytes()).unwrap()
}

/// A `RECIPES` document with one recipe holding `fields`.
fn recipe_doc(root_attrs: &str, fields: &str) -> String {
    format!("<RECIPES{root_attrs}><RECIPE><NAME>Test</NAME>{fields}</RECIPE></RECIPES>")
}

fn warning(path: &str, kind: WarningKind) -> Warning {
    Warning {
        path: path.to_owned(),
        kind,
    }
}

#[test]
fn padded_numbers_are_read() {
    let r = read(&recipe_doc(
        "",
        "<BATCH_SIZE>30.000000</BATCH_SIZE><OG> 1.043 </OG>",
    ));
    let recipe = &r.document.recipes[0];
    assert_eq!(recipe.batch_size, Some(30.0));
    assert_eq!(recipe.og, Some(1.043));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn trailing_text_after_a_number_is_ignored_with_a_warning() {
    let r = read(&recipe_doc("", "<BOIL_TIME>60 min</BOIL_TIME>"));
    assert_eq!(r.document.recipes[0].boil_time, Some(60.0));
    assert_eq!(
        r.warnings,
        [warning(
            "RECIPES/RECIPE[1]/BOIL_TIME",
            WarningKind::TrailingText {
                text: "60 min".into()
            }
        )]
    );
}

#[test]
fn unreadable_number_is_missing_with_a_warning() {
    let r = read(&recipe_doc("", "<EFFICIENCY>about seventy</EFFICIENCY>"));
    assert_eq!(r.document.recipes[0].efficiency, None);
    assert_eq!(
        r.warnings,
        [warning(
            "RECIPES/RECIPE[1]/EFFICIENCY",
            WarningKind::UnreadableNumber {
                text: "about seventy".into()
            }
        )]
    );
}

#[test]
fn missing_and_empty_optional_fields_are_none() {
    let r = read(&recipe_doc("", "<FG></FG><OG/>"));
    let recipe = &r.document.recipes[0];
    assert_eq!(recipe.og, None);
    assert_eq!(recipe.fg, None);
    assert_eq!(recipe.batch_size, None);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn missing_name_is_reported() {
    let r = read("<RECIPES><RECIPE><OG>1.050</OG></RECIPE></RECIPES>");
    assert_eq!(r.document.recipes[0].name, "");
    assert_eq!(
        r.warnings,
        [warning("RECIPES/RECIPE[1]/NAME", WarningKind::MissingName)]
    );
}

#[test]
fn display_fields_are_ignored() {
    for attrs in ["", V1_1_NS] {
        let r = read(&recipe_doc(
            attrs,
            "<DISPLAY_BATCH_SIZE>5.0 gal</DISPLAY_BATCH_SIZE><BATCH_SIZE>18.93</BATCH_SIZE>",
        ));
        assert_eq!(r.document.recipes[0].batch_size, Some(18.93));
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }
}

#[test]
fn unknown_elements_are_ignored_silently_in_1_0() {
    let r = read(
        "<RECIPES><RECIPE><NAME>A</NAME><VENDOR_FIELD>x</VENDOR_FIELD></RECIPE><script/></RECIPES>",
    );
    assert_eq!(r.version, Version::V1_0);
    assert_eq!(r.document.recipes.len(), 1);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn unknown_elements_are_reported_in_1_1() {
    let r = read(&format!(
        "<RECIPES{V1_1_NS}><RECIPE><NAME>A</NAME><VENDOR_FIELD>x</VENDOR_FIELD></RECIPE><script/></RECIPES>"
    ));
    assert_eq!(r.version, Version::V1_1);
    assert_eq!(
        r.warnings,
        [
            warning(
                "RECIPES/RECIPE[1]/VENDOR_FIELD",
                WarningKind::UnknownElement
            ),
            warning("RECIPES/script", WarningKind::UnknownElement),
        ]
    );
}

#[test]
fn known_but_unmodelled_fields_are_not_unknown_in_1_1() {
    let r = read(&recipe_doc(
        V1_1_NS,
        "<BREWER>Me</BREWER><HOPS/><DATE>2024-01-13</DATE>",
    ));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn records_are_returned_in_document_order_with_indexed_paths() {
    let r = read(
        "<RECIPES>\
           <RECIPE><NAME>First</NAME></RECIPE>\
           <RECIPE><NAME>Second</NAME><OG>x</OG></RECIPE>\
         </RECIPES>",
    );
    let names: Vec<_> = r.document.recipes.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["First", "Second"]);
    assert_eq!(r.warnings[0].path, "RECIPES/RECIPE[2]/OG");
}

#[test]
fn version_1_1_from_namespace() {
    assert_eq!(read(&recipe_doc(V1_1_NS, "")).version, Version::V1_1);
}

#[test]
fn version_1_1_from_version_element() {
    assert_eq!(
        read(&recipe_doc("", "<VERSION>1.1</VERSION>")).version,
        Version::V1_1
    );
    assert_eq!(
        read(&recipe_doc("", "<VERSION>1</VERSION>")).version,
        Version::V1_0
    );
}

#[test]
fn other_top_level_lists_are_accepted() {
    for list in [
        "HOPS",
        "FERMENTABLES",
        "YEASTS",
        "MISCS",
        "WATERS",
        "STYLES",
        "EQUIPMENTS",
        "MASHS",
        "MASH_STEPS",
    ] {
        let r = read(&format!("<{list}></{list}>"));
        assert!(r.document.recipes.is_empty(), "{list}");
    }
}

#[test]
fn utf8_bom_is_accepted() {
    let xml = "\u{FEFF}<?xml version=\"1.0\" encoding=\"UTF-8\"?><RECIPES/>";
    assert!(beerxml::read(xml.as_bytes()).is_ok());
}

#[test]
fn not_xml_is_refused() {
    for input in [
        &b"this is not xml"[..],
        b"<RECIPES><RECIPE>",
        b"",
        b"\xFF\xFE<",
    ] {
        assert!(
            matches!(beerxml::read(input), Err(ReadError::NotXml { .. })),
            "{input:?}"
        );
    }
}

#[test]
fn non_beerxml_root_is_refused() {
    assert_eq!(
        beerxml::read(b"<html><body/></html>"),
        Err(ReadError::NoBeerXmlRoot {
            found: "html".into()
        })
    );
}
