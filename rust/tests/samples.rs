//! Every file under `samples/` reads leniently, and its result is snapshotted
//! so model changes show up as reviewable diffs.

use std::path::{Path, PathBuf};

use beerxml::Version;

fn samples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../samples")
        .canonicalize()
        .unwrap()
}

/// Path relative to `samples/`, with forward slashes on every platform.
fn sample_name(path: &Path) -> String {
    path.strip_prefix(samples_dir())
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn every_sample_reads_and_matches_its_snapshot() {
    insta::glob!("../../samples", "**/*.xml", |path| {
        let bytes = std::fs::read(path).unwrap();
        let read = beerxml::read(&bytes)
            .unwrap_or_else(|e| panic!("{} failed to read: {e}", path.display()));
        insta::assert_debug_snapshot!(read);
    });
}

/// The version each sample is detected as by `tests/validate_samples.py`.
/// A sample missing from this table fails the test, so the table is updated
/// together with `samples/`.
const PYTHON_VERSIONS: &[(&str, Version)] = &[
    ("brewtlery_unetice_realworld_sample.xml", Version::V1_0),
    ("corrected/recipes.xml", Version::V1_0),
    ("latin1_beersmith_sample.xml", Version::V1_0),
    ("migrated/recipes_v1.1.xml", Version::V1_1),
    ("original/equipment.xml", Version::V1_0),
    ("original/grain.xml", Version::V1_0),
    ("original/hops.xml", Version::V1_0),
    ("original/mash.xml", Version::V1_0),
    ("original/misc.xml", Version::V1_0),
    ("original/recipes.xml", Version::V1_0),
    ("original/style.xml", Version::V1_0),
    ("original/water.xml", Version::V1_0),
    ("original/yeast.xml", Version::V1_0),
    ("v1.1_sample.xml", Version::V1_1),
];

#[test]
fn version_detection_agrees_with_python() {
    let mut seen = 0;
    insta::glob!("../../samples", "**/*.xml", |path| {
        let name = sample_name(path);
        let expected = PYTHON_VERSIONS
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("{name} is missing from PYTHON_VERSIONS"))
            .1;
        let read = beerxml::read(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(read.version, expected, "{name}");
        seen += 1;
    });
    assert_eq!(
        seen,
        PYTHON_VERSIONS.len(),
        "a sample listed in PYTHON_VERSIONS no longer exists"
    );
}

#[test]
fn recipes_come_back_in_document_order() {
    let bytes = std::fs::read(samples_dir().join("original/recipes.xml")).unwrap();
    let names: Vec<_> = beerxml::read(&bytes)
        .unwrap()
        .document
        .recipes
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(names.first().map(String::as_str), Some("Burton Ale"));
    assert!(names.len() > 1, "expected several recipes, got {names:?}");
}
