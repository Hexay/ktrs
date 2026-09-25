//! Differential test against the compiler's PSI on the parser fixtures (testdata/kotlin/psi), parsed both
//! by file name and as scripts (how ktfmt parses). tests/data/*.hashes hold the JVM oracle's report hash
//! per file; regenerate them with
//!   tools/psi-accessors/psi-accessors.sh hashes testdata/kotlin/psi --fixture > crates/ktrs_psi/tests/data/fixtures.hashes
//!   tools/psi-accessors/psi-accessors.sh hashes testdata/kotlin/psi --fixture --script > crates/ktrs_psi/tests/data/fixtures-script.hashes
//! and inspect a mismatch by diffing `psi-accessors.sh one <file> --fixture [--script]` with
//! `cargo run -p ktrs_psi --release --example psi_accessors -- one <file> --fixture [--script]`.

#[path = "../examples/psi_accessors/report/mod.rs"]
mod report;

use std::path::Path;

fn check(hashes_file: &str, script: bool) {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = crate_dir.join("../../testdata/kotlin/psi");
    let expected = std::fs::read_to_string(crate_dir.join("tests/data").join(hashes_file)).unwrap();
    let actual = report::hashes(&fixtures, true, script);
    assert!(actual.len() >= 680, "expected the 680 parser fixtures, found {}", actual.len());
    let mismatched = report::mismatches(&expected, &actual);
    assert!(mismatched.is_empty(), "{} fixture(s) differ from the JVM PSI:\n{}", mismatched.len(), mismatched.join("\n"));
}

#[test]
fn fixtures_match_jvm_psi() {
    check("fixtures.hashes", false);
}

#[test]
fn fixtures_as_scripts_match_jvm_psi() {
    check("fixtures-script.hashes", true);
}
