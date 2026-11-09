//! Untrusted uploads: entity expansion and external entities can't hurt the
//! reader, and multi-MB files read quickly.

use std::fmt::Write as _;
use std::io::Write as _;
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

use beerxml::ReadError;

/// Reads `bytes` and fails the test if that takes longer than `budget`.
///
/// The read runs on its own thread so a runaway read fails the test at the
/// deadline instead of hanging it.
fn read_within(bytes: &[u8], budget: Duration) -> Result<beerxml::Read, ReadError> {
    let bytes = bytes.to_vec();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(beerxml::read(&bytes));
    });
    rx.recv_timeout(budget)
        .unwrap_or_else(|e| panic!("reading did not finish within {budget:?}: {e}"))
}

fn assert_not_xml(result: Result<beerxml::Read, ReadError>) {
    match result {
        Err(ReadError::NotXml { .. }) => {}
        other => panic!("expected ReadError::NotXml, got {other:?}"),
    }
}

#[test]
fn billion_laughs_is_refused_promptly() {
    let mut xml = String::from("<?xml version=\"1.0\"?>\n<!DOCTYPE HOPS [\n");
    xml.push_str("  <!ENTITY lol0 \"lol\">\n");
    for i in 1..=9 {
        let prev = i - 1;
        writeln!(
            xml,
            "  <!ENTITY lol{i} \"{}\">\n",
            format!("&lol{prev};").repeat(10)
        )
        .unwrap();
    }
    xml.push_str("]>\n<HOPS><HOP><NAME>&lol9;</NAME></HOP></HOPS>");

    assert_not_xml(read_within(xml.as_bytes(), Duration::from_secs(1)));
}

#[test]
fn quadratic_blowup_is_refused_promptly() {
    // One large entity referenced many times: no nesting, but still a
    // multiplication of input size.
    let big = "x".repeat(100_000);
    let refs = "&big;".repeat(10_000);
    let xml = format!(
        "<!DOCTYPE HOPS [<!ENTITY big \"{big}\">]><HOPS><HOP><NAME>{refs}</NAME></HOP></HOPS>"
    );

    assert_not_xml(read_within(xml.as_bytes(), Duration::from_secs(1)));
}

/// A local file whose content must never show up in what the reader returns.
fn secret_file() -> (tempfile_path::TempPath, &'static str) {
    const MARKER: &str = "SECRET-MARKER-3f9c1e";
    let path = tempfile_path::TempPath::new("beerxml_hostile_secret.txt");
    std::fs::File::create(&path.0)
        .unwrap()
        .write_all(MARKER.as_bytes())
        .unwrap();
    (path, MARKER)
}

/// A local TCP listener that records whether anybody connected to it.
fn listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/evil.dtd", listener.local_addr().unwrap());
    (listener, url)
}

fn assert_never_connected(listener: &TcpListener) {
    // Any connect() the reader made has completed by now, so it would be
    // sitting in the accept queue.
    match listener.accept() {
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
        Ok((_, peer)) => panic!("reader connected to the listener from {peer}"),
        Err(e) => panic!("accept failed: {e}"),
    }
}

/// Neither an error message nor any read text contains `marker`.
fn assert_marker_absent(result: &Result<beerxml::Read, ReadError>, marker: &str) {
    let shown = format!("{result:?}");
    assert!(!shown.contains(marker), "file content leaked: {shown}");
}

#[test]
fn external_entity_from_a_local_file_is_never_read() {
    let (secret, marker) = secret_file();
    let url = format!(
        "file:///{}",
        secret.0.display().to_string().replace('\\', "/")
    );
    let xml = format!(
        "<?xml version=\"1.0\"?>\n<!DOCTYPE HOPS [<!ENTITY xxe SYSTEM \"{url}\">]>\n\
         <HOPS><HOP><NAME>&xxe;</NAME></HOP></HOPS>"
    );

    let result = beerxml::read(xml.as_bytes());

    assert_marker_absent(&result, marker);
    assert_not_xml(result);
}

#[test]
fn external_entity_over_http_is_never_fetched() {
    let (listener, url) = listener();
    let xml = format!(
        "<!DOCTYPE HOPS [<!ENTITY xxe SYSTEM \"{url}\">]><HOPS><HOP><NAME>&xxe;</NAME></HOP></HOPS>"
    );

    let result = beerxml::read(xml.as_bytes());

    assert_never_connected(&listener);
    assert_not_xml(result);
}

#[test]
fn external_parameter_entity_is_never_fetched() {
    let (listener, url) = listener();
    let xml = format!(
        "<!DOCTYPE HOPS [<!ENTITY % remote SYSTEM \"{url}\"> %remote;]>\
         <HOPS><HOP><NAME>Saaz</NAME></HOP></HOPS>"
    );

    let result = beerxml::read(xml.as_bytes());

    assert_never_connected(&listener);
    assert_not_xml(result);
}

#[test]
fn external_dtd_is_never_fetched() {
    let (listener, url) = listener();
    let xml = format!("<!DOCTYPE HOPS SYSTEM \"{url}\"><HOPS><HOP><NAME>Saaz</NAME></HOP></HOPS>");

    let result = beerxml::read(xml.as_bytes());

    assert_never_connected(&listener);
    assert_not_xml(result);
}

#[test]
fn entity_reference_without_a_declaration_is_refused() {
    let xml = b"<HOPS><HOP><NAME>&xxe;</NAME></HOP></HOPS>";

    assert_not_xml(beerxml::read(xml));
}

#[test]
fn predefined_and_character_references_still_read() {
    let xml = b"<HOPS><HOP><NAME>Fish &amp; Chips &#x17E;&#253;</NAME></HOP></HOPS>";

    let read = beerxml::read(xml).unwrap();

    assert_eq!(read.document.hops[0].name, "Fish & Chips \u{17E}\u{FD}");
}

/// A `HOPS` ingredient library of `count` fully populated hops.
fn hop_library(count: usize) -> Vec<u8> {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<HOPS>\n");
    for i in 0..count {
        writeln!(
            xml,
            "<HOP><NAME>Hop {i}</NAME><VERSION>1</VERSION><ORIGIN>Czech Republic</ORIGIN>\
             <ALPHA>4.5</ALPHA><AMOUNT>0.0283495</AMOUNT><USE>Boil</USE><TIME>60.000</TIME>\
             <NOTES>Generated hop number {i} for a large-file test.</NOTES>\
             <TYPE>Aroma</TYPE><FORM>Pellet</FORM><BETA>3.5</BETA><HSI>25</HSI>\
             <DISPLAY_AMOUNT>1.00 oz</DISPLAY_AMOUNT></HOP>\n"
        )
        .unwrap();
    }
    xml.push_str("</HOPS>\n");
    xml.into_bytes()
}

#[test]
fn multi_megabyte_library_reads_within_budget() {
    let bytes = hop_library(20_000);
    assert!(bytes.len() > 5_000_000, "only {} bytes", bytes.len());

    // Generous enough for an unoptimised debug build on a slow CI runner;
    // a release build reads this in a fraction of it.
    let read = read_within(&bytes, Duration::from_secs(10)).unwrap();

    assert_eq!(read.document.hops.len(), 20_000);
    assert_eq!(read.document.hops[19_999].name, "Hop 19999");
    assert!(read.warnings.is_empty(), "{:?}", &read.warnings[..1]);
}

/// A temporary file path removed on drop, without a `tempfile` dependency.
mod tempfile_path {
    use std::path::PathBuf;

    pub struct TempPath(pub PathBuf);

    impl TempPath {
        pub fn new(name: &str) -> Self {
            let unique = format!("{}_{name}", std::process::id());
            Self(std::env::temp_dir().join(unique))
        }
    }

    impl Drop for TempPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
}
