//! Strict validation of BeerXML 1.1 "Clean XML" (`validate(bytes,
//! Version::V1_1)`): the 1.1 XSD plus the rules of RFC 01.

use std::path::Path;

use beerxml::{Rule, Version, Violation};

const NS: &str = "http://beerxml.com/v1.1";

fn validate(xml: &str) -> Vec<Violation> {
    beerxml::validate(xml.as_bytes(), Version::V1_1).unwrap()
}

fn violation(path: &str, rule: Rule, value: Option<&str>) -> Violation {
    Violation {
        path: path.to_owned(),
        rule,
        value: value.map(str::to_owned),
    }
}

/// A 1.1 hop library holding one valid hop plus `extra` fields.
fn hop(extra: &str) -> String {
    format!(
        "<HOPS xmlns=\"{NS}\"><HOP><NAME>Saaz</NAME><VERSION>1.1</VERSION>\
         <ALPHA>3.5</ALPHA><AMOUNT>0.05</AMOUNT><USE>Boil</USE><TIME>60</TIME>\
         {extra}</HOP></HOPS>"
    )
}

/// A 1.1 recipe library holding one valid recipe plus `extra` fields.
fn recipe(extra: &str) -> String {
    format!(
        "<RECIPES xmlns=\"{NS}\"><RECIPE><NAME>Pils</NAME><VERSION>1.1</VERSION>\
         <TYPE>All Grain</TYPE><BREWER>Me</BREWER><BATCH_SIZE>20</BATCH_SIZE>\
         <BOIL_SIZE>25</BOIL_SIZE><BOIL_TIME>60</BOIL_TIME>{extra}</RECIPE></RECIPES>"
    )
}

/// A 1.1 yeast library holding one valid yeast plus `extra` fields.
fn yeast(extra: &str) -> String {
    format!(
        "<YEASTS xmlns=\"{NS}\"><YEAST><NAME>W-34/70</NAME><VERSION>1.1</VERSION>\
         <TYPE>Lager</TYPE><FORM>Dry</FORM><AMOUNT>0.011</AMOUNT>{extra}</YEAST></YEASTS>"
    )
}

#[test]
fn a_clean_record_is_valid() {
    assert_eq!(validate(&hop("")), []);
    assert_eq!(validate(&recipe("")), []);
    assert_eq!(validate(&yeast("")), []);
}

#[test]
fn lists_may_be_empty() {
    // Every list in the 1.1 XSD has `minOccurs="0"` records.
    assert_eq!(validate(&format!("<HOPS xmlns=\"{NS}\"/>")), []);
    assert_eq!(validate(&recipe("<HOPS/><MISCS/>")), []);
}

// RFC 01, section 1: numeric fields hold only raw numbers.

#[test]
fn a_unit_inside_a_numeric_field_is_a_violation() {
    assert_eq!(
        validate(&recipe("<IBU>34.2 IBU</IBU>")),
        [violation(
            "RECIPES/RECIPE[1]/IBU",
            Rule::Decimal,
            Some("34.2 IBU")
        )]
    );
    assert_eq!(
        validate(&hop("<BETA>4.5%</BETA>")),
        [violation("HOPS/HOP[1]/BETA", Rule::Decimal, Some("4.5%"))]
    );
    assert_eq!(validate(&recipe("<IBU>34.2</IBU>")), []);
}

#[test]
fn fields_that_were_text_in_1_0_are_numbers_in_1_1() {
    for field in ["EST_ABV", "ABV", "ACTUAL_EFFICIENCY", "CALORIES", "IBUS"] {
        assert_eq!(
            validate(&recipe(&format!("<{field}>5 %</{field}>"))),
            [violation(
                &format!("RECIPES/RECIPE[1]/{field}"),
                Rule::Decimal,
                Some("5 %")
            )],
            "{field}"
        );
    }
}

// RFC 01, section 2: no display hints.

#[test]
fn a_display_field_is_a_violation() {
    assert_eq!(
        validate(&hop("<DISPLAY_AMOUNT>50 g</DISPLAY_AMOUNT>")),
        [violation(
            "HOPS/HOP[1]/DISPLAY_AMOUNT",
            Rule::DisplayField,
            Some("50 g")
        )]
    );
    assert_eq!(
        validate(&recipe(
            "<DISPLAY_OG>1.050 SG</DISPLAY_OG><MASH><NAME>M</NAME><VERSION>1.1</VERSION>\
             <GRAIN_TEMP>20</GRAIN_TEMP><MASH_STEPS/><DISPLAY_GRAIN_TEMP>68 F</DISPLAY_GRAIN_TEMP>\
             </MASH>"
        )),
        [
            violation(
                "RECIPES/RECIPE[1]/DISPLAY_OG",
                Rule::DisplayField,
                Some("1.050 SG")
            ),
            violation(
                "RECIPES/RECIPE[1]/MASH/DISPLAY_GRAIN_TEMP",
                Rule::DisplayField,
                Some("68 F")
            ),
        ]
    );
}

#[test]
fn other_unknown_elements_are_allowed() {
    // As in 1.0 and the Python validator; the reader reports them in 1.1.
    assert_eq!(validate(&hop("<INVENTORY>1.00 oz</INVENTORY>")), []);
}

// RFC 01, section 3: ISO 8601 dates.

#[test]
fn a_date_must_be_iso() {
    assert_eq!(validate(&recipe("<DATE>2004-01-03</DATE>")), []);
    assert_eq!(
        validate(&yeast("<CULTURE_DATE> 2003-07-06 </CULTURE_DATE>")),
        []
    );
    for bad in ["3 Jan 04", "1/3/2004", "2003-02-30", "2004-1-3", ""] {
        assert_eq!(
            validate(&recipe(&format!("<DATE>{bad}</DATE>"))),
            [violation("RECIPES/RECIPE[1]/DATE", Rule::Date, Some(bad))],
            "{bad:?}"
        );
    }
    assert_eq!(
        validate(&yeast("<CULTURE_DATE>6 Jul 03</CULTURE_DATE>")),
        [violation(
            "YEASTS/YEAST[1]/CULTURE_DATE",
            Rule::Date,
            Some("6 Jul 03")
        )]
    );
}

// RFC 01's example and the XSD's `xs:boolean`: lowercase booleans.

#[test]
fn a_boolean_must_be_an_xs_boolean() {
    for good in ["true", "false", "1", "0", " true "] {
        assert_eq!(
            validate(&yeast(&format!(
                "<AMOUNT_IS_WEIGHT>{good}</AMOUNT_IS_WEIGHT>"
            ))),
            [],
            "{good:?}"
        );
    }
    for bad in ["TRUE", "False", "yes", ""] {
        assert_eq!(
            validate(&yeast(&format!(
                "<AMOUNT_IS_WEIGHT>{bad}</AMOUNT_IS_WEIGHT>"
            ))),
            [violation(
                "YEASTS/YEAST[1]/AMOUNT_IS_WEIGHT",
                Rule::LowercaseBoolean,
                Some(bad)
            )],
            "{bad:?}"
        );
    }
}

// RFC 01, section 4: UTF-8 only.

#[test]
fn a_file_declared_in_another_encoding_is_a_violation() {
    let latin1 = format!("<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>{}", hop(""));
    assert_eq!(
        validate(&latin1),
        [violation("HOPS", Rule::Utf8, Some("ISO-8859-1"))]
    );
    let ascii = format!("<?xml version=\"1.0\" encoding=\"US-ASCII\"?>{}", hop(""));
    assert_eq!(
        validate(&ascii),
        [violation("HOPS", Rule::Utf8, Some("US-ASCII"))]
    );
}

#[test]
fn a_utf8_file_is_valid_with_or_without_a_declaration() {
    for decl in [
        "",
        "<?xml version=\"1.0\"?>",
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>",
        "\u{FEFF}<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
    ] {
        assert_eq!(validate(&format!("{decl}{}", hop(""))), [], "{decl:?}");
    }
}

// RFC 01, section 5, as the 1.1 XSD settles it: the namespace and `VERSION`.

#[test]
fn the_root_must_be_in_the_1_1_namespace() {
    let bare = hop("").replace(&format!(" xmlns=\"{NS}\""), "");
    assert_eq!(validate(&bare), [violation("HOPS", Rule::Namespace, None)]);
    let other = hop("").replace(NS, "http://beerxml.com/v1");
    assert_eq!(
        validate(&other),
        [violation(
            "HOPS",
            Rule::Namespace,
            Some("http://beerxml.com/v1")
        )]
    );
    let prefixed = format!(
        "<b:HOPS xmlns:b=\"{NS}\"><b:HOP><b:NAME>Saaz</b:NAME><b:VERSION>1.1</b:VERSION>\
         <b:ALPHA>3.5</b:ALPHA><b:AMOUNT>0.05</b:AMOUNT><b:USE>Boil</b:USE><b:TIME>60</b:TIME>\
         </b:HOP></b:HOPS>"
    );
    assert_eq!(validate(&prefixed), []);
}

#[test]
fn every_record_must_say_version_1_1() {
    assert_eq!(
        validate(&hop("").replace("<VERSION>1.1</VERSION>", "<VERSION>1</VERSION>")),
        [violation("HOPS/HOP[1]/VERSION", Rule::Version, Some("1"))]
    );
    // `xs:decimal` with `fixed="1.1"` compares values, not text.
    for same in ["1.10", "+1.1", " 1.1 ", "01.1"] {
        let xml = hop("").replace(
            "<VERSION>1.1</VERSION>",
            &format!("<VERSION>{same}</VERSION>"),
        );
        assert_eq!(validate(&xml), [], "{same:?}");
    }
    assert_eq!(
        validate(&hop("").replace("<VERSION>1.1</VERSION>", "")),
        [violation("HOPS/HOP[1]/VERSION", Rule::Required, None)]
    );
}

// The checks shared with 1.0 still apply.

#[test]
fn shared_checks_still_apply() {
    assert_eq!(
        validate(&hop("<TYPE>bittering</TYPE><HSI>-1</HSI>").replace("<USE>Boil</USE>", "")),
        [
            violation(
                "HOPS/HOP[1]/TYPE",
                Rule::Enumeration {
                    allowed: &["Bittering", "Aroma", "Both"]
                },
                Some("bittering")
            ),
            violation("HOPS/HOP[1]/HSI", Rule::NonNegative, Some("-1")),
            violation("HOPS/HOP[1]/USE", Rule::Required, None),
        ]
    );
}

#[test]
fn a_violation_reads_as_a_sentence() {
    let shown = |rule| violation("X", rule, Some("v")).to_string();
    assert_eq!(
        shown(Rule::DisplayField),
        "X: \"v\" is a display field, which 1.1 removes"
    );
    assert_eq!(
        shown(Rule::Date),
        "X: \"v\" is not an ISO date (YYYY-MM-DD)"
    );
    assert_eq!(
        shown(Rule::LowercaseBoolean),
        "X: \"v\" is not true, false, 1 or 0"
    );
    assert_eq!(shown(Rule::Utf8), "X: \"v\" is not UTF-8");
    assert_eq!(shown(Rule::Version), "X: \"v\" is not 1.1");
    assert_eq!(
        shown(Rule::Namespace),
        "X: \"v\" is not the namespace http://beerxml.com/v1.1"
    );
    assert_eq!(
        violation("X", Rule::Namespace, None).to_string(),
        "X: root element is not in the namespace http://beerxml.com/v1.1"
    );
}

// Samples.

fn sample(name: &str) -> Vec<Violation> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../samples")
        .join(name);
    beerxml::validate(&std::fs::read(path).unwrap(), Version::V1_1).unwrap()
}

#[test]
fn the_1_1_samples_are_valid() {
    for name in ["v1.1_sample.xml", "migrated/recipes_v1.1.xml"] {
        let violations = sample(name);
        let shown: Vec<_> = violations.iter().take(5).map(ToString::to_string).collect();
        assert!(violations.is_empty(), "{name}: {shown:#?}");
    }
}

#[test]
fn a_1_0_sample_breaks_the_clean_xml_rules() {
    let rules = |name| {
        sample(name)
            .into_iter()
            .map(|v| std::mem::discriminant(&v.rule))
            .collect::<Vec<_>>()
    };
    let has = |rules: &[_], rule: Rule| rules.contains(&std::mem::discriminant(&rule));

    let recipes = rules("original/recipes.xml");
    for rule in [
        Rule::Namespace,
        Rule::Version,
        Rule::DisplayField,
        Rule::Date,
        Rule::LowercaseBoolean,
        Rule::Decimal,
    ] {
        assert!(
            has(&recipes, rule.clone()),
            "original/recipes.xml lacks {rule:?}"
        );
    }
    assert!(has(&rules("latin1_beersmith_sample.xml"), Rule::Utf8));
}
