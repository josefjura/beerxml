//! Character encodings: UTF-8 (with or without a BOM) and ISO-8859-1 read;
//! any other declared encoding is refused.

use std::path::Path;

use beerxml::ReadError;

fn sample(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../samples")
            .join(name),
    )
    .unwrap()
}

/// A `HOPS` document with one hop named `name`, after `prolog`. `name` is
/// Latin-1 encoded, so it may only contain characters up to U+00FF.
fn latin1_doc(prolog: &str, name: &str) -> Vec<u8> {
    let mut bytes = prolog.as_bytes().to_vec();
    bytes.extend_from_slice(b"<HOPS><HOP><NAME>");
    bytes.extend(name.chars().map(|c| u8::try_from(u32::from(c)).unwrap()));
    bytes.extend_from_slice(b"</NAME></HOP></HOPS>");
    bytes
}

fn hop_name(bytes: &[u8]) -> String {
    let read = beerxml::read(bytes).unwrap();
    read.document.hops[0].name.clone()
}

fn unsupported(encoding: &str) -> ReadError {
    ReadError::UnsupportedEncoding {
        encoding: encoding.to_owned(),
    }
}

#[test]
fn latin1_file_reads_with_names_intact() {
    let bytes = latin1_doc(
        r#"<?xml version="1.0" encoding="ISO-8859-1"?>"#,
        "Weißbier Mittelfrüh Údolí",
    );
    assert_eq!(hop_name(&bytes), "Weißbier Mittelfrüh Údolí");
}

#[test]
fn every_latin1_byte_is_its_own_code_point() {
    // Bracketed by `[` `]` so that trimming the name keeps U+00A0.
    let mut bytes = br#"<?xml version="1.0" encoding="ISO-8859-1"?><HOPS><HOP><NAME>["#.to_vec();
    bytes.extend(0xA0..=0xFF_u8);
    bytes.extend_from_slice(b"]</NAME></HOP></HOPS>");
    let expected: String = std::iter::once('[')
        .chain((0xA0..=0xFF_u8).map(char::from))
        .chain(std::iter::once(']'))
        .collect();
    assert_eq!(hop_name(&bytes), expected);
}

#[test]
fn latin1_file_can_carry_other_characters_as_references() {
    let bytes = latin1_doc(
        r#"<?xml version="1.0" encoding="ISO-8859-1"?>"#,
        "Ún&#283;tický",
    );
    assert_eq!(hop_name(&bytes), "Únětický");
}

#[test]
fn latin1_sample_reads_with_names_intact() {
    let read = beerxml::read(&sample("latin1_beersmith_sample.xml")).unwrap();
    let recipe = &read.document.recipes[0];
    assert_eq!(recipe.name, "Bayerisches Weißbier");
    assert_eq!(recipe.brewer.as_deref(), Some("Jörg Müller"));
    assert_eq!(recipe.asst_brewer.as_deref(), Some("Tomáš Únětický"));
    let hops: Vec<_> = recipe.hops.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(
        hops,
        ["Hallertauer Mittelfrüh", "Žatecký poloraný červeňák"]
    );
    assert_eq!(recipe.yeasts[0].name, "Weißbier-Hefe");
}

#[test]
fn latin1_declaration_is_matched_case_insensitively_with_aliases() {
    for encoding in [
        "ISO-8859-1",
        "iso-8859-1",
        "Iso-8859-1",
        "ISO8859-1",
        "ISO_8859-1",
        "ISO_8859-1:1987",
        "latin1",
        "Latin-1",
        "L1",
        "ISO-IR-100",
        "IBM819",
        "CP819",
        "csISOLatin1",
    ] {
        let bytes = latin1_doc(
            &format!("<?xml version='1.0' encoding='{encoding}'?>"),
            "Weißbier",
        );
        assert_eq!(hop_name(&bytes), "Weißbier", "{encoding}");
    }
}

#[test]
fn declaration_tolerates_whitespace_around_the_equals_sign() {
    let bytes = latin1_doc(
        "<?xml version = \"1.0\"\n  encoding = \"iso-8859-1\" standalone=\"yes\" ?>",
        "Weißbier",
    );
    assert_eq!(hop_name(&bytes), "Weißbier");
}

#[test]
fn utf8_reads_declared_or_not_with_or_without_bom() {
    let body = "<HOPS><HOP><NAME>Únětický Weißbier</NAME></HOP></HOPS>";
    for prolog in [
        "",
        r#"<?xml version="1.0"?>"#,
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<?xml version="1.0" encoding="utf-8"?>"#,
        r#"<?xml version="1.0" encoding="UTF8"?>"#,
    ] {
        for bom in [&b""[..], b"\xEF\xBB\xBF"] {
            let mut bytes = bom.to_vec();
            bytes.extend_from_slice(prolog.as_bytes());
            bytes.extend_from_slice(body.as_bytes());
            assert_eq!(hop_name(&bytes), "Únětický Weißbier", "{prolog:?} {bom:?}");
        }
    }
}

#[test]
fn ascii_declaration_reads_as_utf8() {
    let bytes =
        br#"<?xml version="1.0" encoding="US-ASCII"?><HOPS><HOP><NAME>Saaz</NAME></HOP></HOPS>"#;
    assert_eq!(hop_name(bytes), "Saaz");
}

#[test]
fn other_declared_encodings_are_refused_by_name() {
    for encoding in [
        "windows-1250",
        "UTF-16",
        "windows-1252",
        "ISO-8859-2",
        "Shift_JIS",
    ] {
        let xml = format!(
            r#"<?xml version="1.0" encoding="{encoding}"?><HOPS><HOP><NAME>Saaz</NAME></HOP></HOPS>"#
        );
        assert_eq!(
            beerxml::read(xml.as_bytes()),
            Err(unsupported(encoding)),
            "{encoding}"
        );
    }
}

#[test]
fn utf16_byte_order_marks_are_refused() {
    let xml = "<?xml version=\"1.0\"?><HOPS/>";
    let le: Vec<u8> = [0xFF, 0xFE]
        .into_iter()
        .chain(xml.encode_utf16().flat_map(u16::to_le_bytes))
        .collect();
    let be: Vec<u8> = [0xFE, 0xFF]
        .into_iter()
        .chain(xml.encode_utf16().flat_map(u16::to_be_bytes))
        .collect();
    assert_eq!(beerxml::read(&le), Err(unsupported("UTF-16LE")));
    assert_eq!(beerxml::read(&be), Err(unsupported("UTF-16BE")));
}

#[test]
fn utf16_without_a_byte_order_mark_is_refused() {
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-16\"?><HOPS/>";
    let le: Vec<u8> = xml.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let be: Vec<u8> = xml.encode_utf16().flat_map(u16::to_be_bytes).collect();
    assert_eq!(beerxml::read(&le), Err(unsupported("UTF-16LE")));
    assert_eq!(beerxml::read(&be), Err(unsupported("UTF-16BE")));
}

#[test]
fn latin1_bytes_without_a_latin1_declaration_are_not_xml() {
    let bytes = latin1_doc(r#"<?xml version="1.0" encoding="UTF-8"?>"#, "Weißbier");
    assert!(matches!(
        beerxml::read(&bytes),
        Err(ReadError::NotXml { .. })
    ));
    let bytes = latin1_doc("", "Weißbier");
    assert!(matches!(
        beerxml::read(&bytes),
        Err(ReadError::NotXml { .. })
    ));
}

#[test]
fn unsupported_encoding_error_names_the_encoding() {
    assert_eq!(
        unsupported("windows-1250").to_string(),
        "unsupported character encoding \"windows-1250\": only UTF-8 and ISO-8859-1 are read"
    );
}
