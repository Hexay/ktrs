//! `ktrs ktlint`: the `ktlint` drop-in behind the `ktrs` binary (what the Gradle plugin runs).

mod common;

use common::{TempDir, strings, write_text};

#[test]
fn runs_the_ktlint_drop_in() {
    let dir = TempDir::new("ktrs-ktlint-passthrough");
    let clean = dir.path().join("Clean.kt");
    let failing = dir.path().join("Failing.kt");
    write_text(&clean, "val foo = \"bar\"\n");
    write_text(&failing, "val  foo = \"bar\"\n");
    let report = dir.path().join("report.json");
    let run = |file: &std::path::Path| {
        let output = format!("--reporter=json,output={}", report.display());
        ktrs_cli::ktrs::run(&strings(&["ktlint", "--ktlint-version=1.8", &output, &file.display().to_string()]))
    };
    assert_eq!(run(&clean), 0);
    assert_eq!(run(&failing), 1);
    assert!(std::fs::read_to_string(&report).unwrap().contains("standard:no-multi-spaces"));
}
