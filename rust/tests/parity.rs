//! Strict validation agrees with the Python reference validator on every
//! sample.
//!
//! `fixtures/python_verdicts.tsv` records whether `lib/validator.py` accepts
//! each sample under each version; regenerate it with
//! `python3 scripts/python_verdicts.py`. Only 1.0 rows are asserted until
//! strict 1.1 validation exists (issue #10).
//!
//! Known disagreements on files outside `samples/` (the Python validator is
//! looser than the XSD; Rust follows the XSD):
//!
//! - Python strips whitespace before checking enumerations and booleans, so
//!   it accepts `Boil ` and ` TRUE`.
//! - Python skips empty elements, so it accepts `<ALPHA></ALPHA>`.
//! - Python reads numbers with `float()`/`int()`, so it accepts `1e3`, `NaN`,
//!   `inf` and `1_000`, and does not check the XSD's `minInclusive="0"`.

use std::path::Path;

use beerxml::Version;

const TABLE: &str = include_str!("fixtures/python_verdicts.tsv");

struct Verdict<'a> {
    sample: &'a str,
    version: &'a str,
    valid: bool,
}

fn verdicts() -> Vec<Verdict<'static>> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|line| {
            let cols: Vec<_> = line.split('\t').collect();
            let [sample, version, verdict, _errors] = cols[..] else {
                panic!("malformed line {line:?}");
            };
            Verdict {
                sample,
                version,
                valid: match verdict {
                    "valid" => true,
                    "invalid" => false,
                    other => panic!("unknown verdict {other:?}"),
                },
            }
        })
        .collect()
}

fn sample_name(path: &Path) -> String {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../samples")
        .canonicalize()
        .unwrap();
    path.strip_prefix(samples)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn every_sample_has_a_python_verdict_for_each_version() {
    let table = verdicts();
    let mut samples = 0;
    insta::glob!("../../samples", "**/*.xml", |path| {
        let name = sample_name(path);
        for version in ["1.0", "1.1"] {
            let rows = table
                .iter()
                .filter(|v| v.sample == name && v.version == version)
                .count();
            assert_eq!(
                rows, 1,
                "{name} needs one {version} row: run python3 scripts/python_verdicts.py"
            );
        }
        samples += 1;
    });
    assert_eq!(table.len(), samples * 2, "the table lists a removed sample");
}

#[test]
fn validate_1_0_agrees_with_python_on_every_sample() {
    let table = verdicts();
    insta::glob!("../../samples", "**/*.xml", |path| {
        let name = sample_name(path);
        let python = table
            .iter()
            .find(|v| v.sample == name && v.version == "1.0")
            .unwrap_or_else(|| panic!("{name} has no 1.0 verdict"));
        let violations = beerxml::validate(&std::fs::read(path).unwrap(), Version::V1_0)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let shown: Vec<_> = violations.iter().take(5).map(ToString::to_string).collect();
        assert_eq!(
            violations.is_empty(),
            python.valid,
            "{name}: Python says {}, Rust found {} violations, e.g. {shown:#?}",
            if python.valid { "valid" } else { "invalid" },
            violations.len(),
        );
    });
}
