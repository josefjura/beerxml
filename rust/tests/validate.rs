//! Strict validation of BeerXML 1.0 (`validate(bytes, Version::V1_0)`).

use std::fmt::Write;

use beerxml::{ReadError, Rule, Version, Violation};

fn validate(xml: &str) -> Vec<Violation> {
    beerxml::validate(xml.as_bytes(), Version::V1_0).unwrap()
}

fn violation(path: &str, rule: Rule, value: Option<&str>) -> Violation {
    Violation {
        path: path.to_owned(),
        rule,
        value: value.map(str::to_owned),
    }
}

fn missing(path: &str) -> Violation {
    violation(path, Rule::Required, None)
}

/// A record type: its list element, its record element and a record holding
/// exactly its required fields (each valid), as `(element, content)` pairs.
struct RecordType {
    list: &'static str,
    record: &'static str,
    required: &'static [(&'static str, &'static str)],
}

const MASH_STEP_XML: &str = "<MASH_STEP><NAME>Rest</NAME><VERSION>1</VERSION><TYPE>Infusion</TYPE>\
     <STEP_TEMP>66</STEP_TEMP><STEP_TIME>60</STEP_TIME></MASH_STEP>";

const RECORD_TYPES: &[RecordType] = &[
    RecordType {
        list: "HOPS",
        record: "HOP",
        required: &[
            ("NAME", "Saaz"),
            ("VERSION", "1"),
            ("ALPHA", "3.5"),
            ("AMOUNT", "0.05"),
            ("USE", "Boil"),
            ("TIME", "60"),
        ],
    },
    RecordType {
        list: "FERMENTABLES",
        record: "FERMENTABLE",
        required: &[
            ("NAME", "Pilsner"),
            ("VERSION", "1"),
            ("TYPE", "Grain"),
            ("AMOUNT", "5"),
            ("YIELD", "80"),
            ("COLOR", "2"),
        ],
    },
    RecordType {
        list: "YEASTS",
        record: "YEAST",
        required: &[
            ("NAME", "W-34/70"),
            ("VERSION", "1"),
            ("TYPE", "Lager"),
            ("FORM", "Dry"),
            ("AMOUNT", "0.011"),
        ],
    },
    RecordType {
        list: "MISCS",
        record: "MISC",
        required: &[
            ("NAME", "Irish Moss"),
            ("VERSION", "1"),
            ("TYPE", "Fining"),
            ("USE", "Boil"),
            ("TIME", "15"),
            ("AMOUNT", "0.01"),
        ],
    },
    RecordType {
        list: "WATERS",
        record: "WATER",
        required: &[("NAME", "Pilsen"), ("VERSION", "1"), ("AMOUNT", "20")],
    },
    RecordType {
        list: "EQUIPMENTS",
        record: "EQUIPMENT",
        required: &[
            ("NAME", "Kettle"),
            ("VERSION", "1"),
            ("BOIL_SIZE", "25"),
            ("BATCH_SIZE", "20"),
        ],
    },
    RecordType {
        list: "STYLES",
        record: "STYLE",
        required: &[
            ("NAME", "Pilsner"),
            ("CATEGORY", "Lager"),
            ("VERSION", "1"),
            ("TYPE", "Lager"),
            ("OG_MIN", "1.044"),
            ("OG_MAX", "1.050"),
            ("FG_MIN", "1.008"),
            ("FG_MAX", "1.013"),
            ("IBU_MIN", "25"),
            ("IBU_MAX", "45"),
            ("COLOR_MIN", "2"),
            ("COLOR_MAX", "5"),
        ],
    },
    RecordType {
        list: "MASH_STEPS",
        record: "MASH_STEP",
        required: &[
            ("NAME", "Rest"),
            ("VERSION", "1"),
            ("TYPE", "Infusion"),
            ("STEP_TEMP", "66"),
            ("STEP_TIME", "60"),
        ],
    },
    RecordType {
        list: "MASHS",
        record: "MASH",
        required: &[
            ("NAME", "Single infusion"),
            ("VERSION", "1"),
            ("GRAIN_TEMP", "20"),
            ("MASH_STEPS", MASH_STEP_XML),
        ],
    },
    RecordType {
        list: "RECIPES",
        record: "RECIPE",
        required: &[
            ("NAME", "Pils"),
            ("VERSION", "1"),
            ("TYPE", "All Grain"),
            ("BREWER", "Josef"),
            ("BATCH_SIZE", "20"),
            ("BOIL_SIZE", "25"),
            ("BOIL_TIME", "60"),
        ],
    },
];

/// A one-record document of `t`, leaving out the required field `skip`.
fn document(t: &RecordType, skip: Option<&str>) -> String {
    let mut fields = String::new();
    for (name, content) in t.required.iter().filter(|(name, _)| Some(*name) != skip) {
        write!(fields, "<{name}>{content}</{name}>").unwrap();
    }
    format!(
        "<{list}><{record}>{fields}</{record}></{list}>",
        list = t.list,
        record = t.record
    )
}

/// A `HOPS` document with one valid hop plus `extra` fields.
fn hop(extra: &str) -> String {
    format!(
        "<HOPS><HOP><NAME>Saaz</NAME><VERSION>1</VERSION><ALPHA>3.5</ALPHA>\
         <AMOUNT>0.05</AMOUNT><USE>Boil</USE><TIME>60</TIME>{extra}</HOP></HOPS>"
    )
}

#[test]
fn a_record_with_every_required_field_is_valid() {
    for t in RECORD_TYPES {
        assert_eq!(validate(&document(t, None)), [], "{}", t.record);
    }
}

#[test]
fn a_hop_without_alpha_is_a_violation() {
    let xml = hop("").replace("<ALPHA>3.5</ALPHA>", "");
    assert_eq!(validate(&xml), [missing("HOPS/HOP[1]/ALPHA")]);
}

#[test]
fn every_required_field_of_every_record_type_is_checked() {
    for t in RECORD_TYPES {
        for (name, _) in t.required {
            let path = format!("{}/{}[1]/{name}", t.list, t.record);
            assert_eq!(validate(&document(t, Some(name))), [missing(&path)]);
        }
    }
}

#[test]
fn optional_fields_may_be_left_out() {
    // ADD_AFTER_BOIL is optional in a FERMENTABLE, NOTES in every record.
    assert_eq!(validate(&hop("<NOTES>Noble</NOTES>")), []);
}

#[test]
fn a_list_needs_its_records_where_the_schema_says_so() {
    assert_eq!(validate("<HOPS/>"), [missing("HOPS/HOP")]);
    assert_eq!(
        validate("<MASH_STEPS></MASH_STEPS>"),
        [missing("MASH_STEPS/MASH_STEP")]
    );
    // An empty recipe or water list is valid.
    assert_eq!(validate("<RECIPES/>"), []);
    assert_eq!(validate("<WATERS/>"), []);
}

#[test]
fn records_inside_a_recipe_are_checked_with_their_index() {
    let recipe = document(&RECORD_TYPES[9], None).replace(
        "</RECIPE>",
        "<HOPS><HOP><NAME>A</NAME></HOP><HOP><NAME>B</NAME><VERSION>1</VERSION><ALPHA>4</ALPHA>\
         <AMOUNT>0.01</AMOUNT><USE>Aroma</USE></HOP></HOPS>\
         <MASH><NAME>M</NAME><VERSION>1</VERSION><GRAIN_TEMP>20</GRAIN_TEMP></MASH>\
         <STYLE><NAME>S</NAME></STYLE></RECIPE>",
    );
    let violations = validate(&recipe);
    for path in [
        "RECIPES/RECIPE[1]/HOPS/HOP[1]/ALPHA",
        "RECIPES/RECIPE[1]/HOPS/HOP[2]/TIME",
        "RECIPES/RECIPE[1]/MASH/MASH_STEPS",
        "RECIPES/RECIPE[1]/STYLE/CATEGORY",
    ] {
        assert!(
            violations.contains(&missing(path)),
            "{path}: {violations:?}"
        );
    }
}

#[test]
fn a_decimal_field_must_hold_a_decimal() {
    for text in ["5.5%", "", "abc", "1e3", "NaN", "1,5"] {
        assert_eq!(
            validate(&hop(&format!("<BETA>{text}</BETA>"))),
            [violation("HOPS/HOP[1]/BETA", Rule::Decimal, Some(text))],
            "{text:?}"
        );
    }
    for text in ["4", "4.", ".5", "+4.25", " 4.5\n"] {
        assert_eq!(
            validate(&hop(&format!("<BETA>{text}</BETA>"))),
            [],
            "{text:?}"
        );
    }
}

#[test]
fn a_non_negative_field_must_not_be_negative() {
    assert_eq!(
        validate(&hop("<BETA>-1.5</BETA>")),
        [violation(
            "HOPS/HOP[1]/BETA",
            Rule::NonNegative,
            Some("-1.5")
        )]
    );
    assert_eq!(validate(&hop("<BETA>-0.0</BETA>")), []);
    // A temperature may be below zero.
    let yeast = document(&RECORD_TYPES[2], None)
        .replace("</YEAST>", "<MIN_TEMPERATURE>-2</MIN_TEMPERATURE></YEAST>");
    assert_eq!(validate(&yeast), []);
}

#[test]
fn an_integer_field_must_hold_an_integer() {
    for text in ["1.0", "one", "", "99999999999"] {
        let xml = hop("").replace(
            "<VERSION>1</VERSION>",
            &format!("<VERSION>{text}</VERSION>"),
        );
        assert_eq!(
            validate(&xml),
            [violation("HOPS/HOP[1]/VERSION", Rule::Integer, Some(text))],
            "{text:?}"
        );
    }
    let yeast = document(&RECORD_TYPES[2], None)
        .replace("</YEAST>", "<TIMES_CULTURED>-3</TIMES_CULTURED></YEAST>");
    assert_eq!(validate(&yeast), []);
}

#[test]
fn a_boolean_field_must_be_upper_case_true_or_false() {
    let fermentable = |text: &str| {
        document(&RECORD_TYPES[1], None).replace(
            "</FERMENTABLE>",
            &format!("<ADD_AFTER_BOIL>{text}</ADD_AFTER_BOIL></FERMENTABLE>"),
        )
    };
    for text in ["TRUE", "FALSE"] {
        assert_eq!(validate(&fermentable(text)), [], "{text:?}");
    }
    for text in ["true", "1", "yes", " TRUE"] {
        assert_eq!(
            validate(&fermentable(text)),
            [violation(
                "FERMENTABLES/FERMENTABLE[1]/ADD_AFTER_BOIL",
                Rule::Boolean,
                Some(text)
            )],
            "{text:?}"
        );
    }
}

#[test]
fn an_enumerated_field_must_match_the_spec_list_exactly() {
    let allowed: &[&str] = &["Boil", "Dry Hop", "Mash", "First Wort", "Aroma"];
    for text in ["Dry hop ", "Dry Hop ", "boil", "Whirlpool"] {
        let xml = hop("").replace("<USE>Boil</USE>", &format!("<USE>{text}</USE>"));
        assert_eq!(
            validate(&xml),
            [violation(
                "HOPS/HOP[1]/USE",
                Rule::Enumeration { allowed },
                Some(text)
            )],
            "{text:?}"
        );
    }
    assert_eq!(validate(&hop("<FORM>Pellet</FORM>")), []);
}

#[test]
fn unknown_and_display_elements_are_allowed() {
    // BeerXML 1.0 lets writers add their own elements.
    assert_eq!(
        validate(&hop(
            "<DISPLAY_AMOUNT>50 g</DISPLAY_AMOUNT><X_VENDOR>1</X_VENDOR>"
        )),
        []
    );
}

#[test]
fn namespaces_are_ignored_like_the_reader_does() {
    let xml = hop("").replace("<HOPS>", "<HOPS xmlns=\"http://example.com/x\">");
    assert_eq!(validate(&xml), []);
}

#[test]
fn every_violation_is_reported_in_one_pass() {
    let xml = hop("<BETA>x</BETA><FORM>Cone</FORM>").replace("<TIME>60</TIME>", "");
    let rules: Vec<_> = validate(&xml).into_iter().map(|v| v.rule).collect();
    assert_eq!(rules.len(), 3, "{rules:?}");
}

#[test]
fn a_violation_reads_as_a_sentence() {
    let v = violation("HOPS/HOP[1]/BETA", Rule::Decimal, Some("5.5%"));
    assert_eq!(
        v.to_string(),
        "HOPS/HOP[1]/BETA: \"5.5%\" is not a decimal number"
    );
    assert_eq!(
        missing("HOPS/HOP[1]/ALPHA").to_string(),
        "HOPS/HOP[1]/ALPHA: required element is missing"
    );
}

#[test]
fn fatal_problems_are_still_read_errors() {
    let err = |xml: &[u8]| beerxml::validate(xml, Version::V1_0).unwrap_err();
    assert!(matches!(err(b"not xml"), ReadError::NotXml { .. }));
    assert!(matches!(err(b"<FOO/>"), ReadError::NoBeerXmlRoot { .. }));
    assert!(matches!(
        err(b"<?xml version=\"1.0\" encoding=\"windows-1250\"?><HOPS/>"),
        ReadError::UnsupportedEncoding { .. }
    ));
}

#[test]
fn version_1_1_is_not_validated_yet() {
    assert_eq!(
        beerxml::validate(b"<HOPS/>", Version::V1_1),
        Err(ReadError::UnsupportedVersion {
            version: Version::V1_1
        })
    );
}
