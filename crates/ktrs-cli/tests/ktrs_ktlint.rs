//! `ktrs ktlint`: the `ktlint` drop-in behind the `ktrs` binary, and the Gradle plugin's hidden options
//! (`crates/ktrs-cli/src/ktlint/gradle.rs`).

mod common;

use std::path::Path;

use common::{TempDir, strings, write_text};

fn ktlint(args: &[&str]) -> i32 {
    let mut all = vec!["ktlint", "--ktlint-version=1.8"];
    all.extend_from_slice(args);
    ktrs_cli::ktrs::run(&strings(&all))
}

fn path(p: &Path) -> String {
    p.display().to_string()
}

#[test]
fn runs_the_ktlint_drop_in() {
    let dir = TempDir::new("ktrs-ktlint-passthrough");
    let clean = dir.path().join("Clean.kt");
    let failing = dir.path().join("Failing.kt");
    write_text(&clean, "val foo = \"bar\"\n");
    write_text(&failing, "val  foo = \"bar\"\n");
    let output = format!("--reporter=json,output={}", path(&dir.path().join("report.json")));
    assert_eq!(ktlint(&[&output, &path(&clean)]), 0);
    assert_eq!(ktlint(&[&output, &path(&failing)]), 1);
    assert!(std::fs::read_to_string(dir.path().join("report.json")).unwrap().contains("standard:no-multi-spaces"));
}

#[test]
fn gradle_format_reports_every_error_with_its_status_and_whether_it_was_fixed() {
    let dir = TempDir::new("ktrs-ktlint-gradle-format");
    let file = dir.path().join("A.kt");
    write_text(&file, "import java.util.*\n\nval  foo = 1\n");
    let events = dir.path().join("events.txt");
    let plain = dir.path().join("plain.txt");
    let code = ktlint(&[
        "--format",
        &format!("--ktrs-gradle-events={}", path(&events)),
        &format!("--reporter=plain,output={}", path(&plain)),
        &path(&file),
    ]);
    assert_eq!(code, 1, "the wildcard import is left");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "import java.util.*\n\nval foo = 1\n");
    let events = std::fs::read_to_string(&events).unwrap();
    let rows: Vec<&str> = events.lines().skip(1).map(|l| l.split_once('\t').unwrap().0).collect();
    assert_eq!(rows, ["file", "error", "error"]);
    assert!(events.contains("\t1\t1\tstandard:no-wildcard-imports\tLINT_CAN_NOT_BE_AUTOCORRECTED\tfalse\tWildcard import\n"));
    assert!(
        events.contains("\t3\t5\tstandard:no-multi-spaces\tLINT_CAN_BE_AUTOCORRECTED\ttrue\tUnnecessary long whitespace\n"),
        "{events}"
    );
    let plain = std::fs::read_to_string(&plain).unwrap();
    assert!(plain.contains("1:1: Wildcard import (standard:no-wildcard-imports)"), "plain detail, no suffix: {plain}");
    assert!(plain.contains("3:5: Unnecessary long whitespace"), "fixed errors are reported too: {plain}");
}

#[test]
fn gradle_editorconfig_override_wins_over_the_editorconfig() {
    let dir = TempDir::new("ktrs-ktlint-gradle-override");
    write_text(&dir.path().join(".editorconfig"), "root = true\n[*.kt]\nmax_line_length = 200\n");
    let file = dir.path().join("A.kt");
    write_text(&file, "val foo = \"a long enough line\"\n");
    assert_eq!(ktlint(&[&path(&file)]), 0);
    assert_eq!(ktlint(&["--ktrs-editorconfig-override=max_line_length=20", &path(&file)]), 1);
    assert_eq!(ktlint(&["--ktrs-editorconfig-override=no_such_property=1", &path(&file)]), 1, "an unknown property fails");
}

#[test]
fn gradle_relative_to_sets_the_base_of_relative_paths() {
    let dir = TempDir::new("ktrs-ktlint-gradle-relative");
    let file = dir.path().join("sub").join("A.kt");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    write_text(&file, "val  foo = 1\n");
    let events = dir.path().join("events.txt");
    let args = [
        "--relative".to_owned(),
        format!("--ktrs-relative-to={}", path(dir.path())),
        format!("--ktrs-gradle-events={}", path(&events)),
        path(&file),
    ];
    assert_eq!(ktlint(&args.iter().map(String::as_str).collect::<Vec<_>>()), 1);
    let expected = format!("file\tsub{}A.kt\n", std::path::MAIN_SEPARATOR);
    assert!(std::fs::read_to_string(&events).unwrap().contains(&expected), "platform separators, as ktlint-gradle's");
}
