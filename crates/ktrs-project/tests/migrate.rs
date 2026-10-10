//! `ktrs migrate`'s rewrites: each fixture's changed files against `fixtures/migrated/<fixture>/` (only the
//! files that change; `UPDATE_MIGRATED=1` rewrites them), its notes, and a second run after writing.

mod common;

use std::path::{Path, PathBuf};

use common::fixture;
use ktrs_project::migrate::{Migration, plan};

const VERSION: &str = "0.5.0";

fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != ".gradle") {
                files_under(&path, out);
            }
        } else {
            out.push(path);
        }
    }
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/")
}

/// Checks `name`'s changes against the expected files and its notes against `notes` (substrings, one each).
fn check(name: &str, notes: &[&str]) -> Migration {
    let root = fixture(name);
    let m = plan(&root, VERSION);
    let expected_dir = fixture("migrated").join(name);
    if std::env::var_os("UPDATE_MIGRATED").is_some() {
        let _ = std::fs::remove_dir_all(&expected_dir);
        for c in &m.changes {
            let out = expected_dir.join(rel(&root, &c.path));
            std::fs::create_dir_all(out.parent().unwrap()).unwrap();
            std::fs::write(out, &c.after).unwrap();
        }
    }
    let mut expected = Vec::new();
    files_under(&expected_dir, &mut expected);
    let mut expected: Vec<String> = expected.iter().map(|p| rel(&expected_dir, p)).collect();
    expected.sort();
    let mut actual: Vec<String> = m.changes.iter().map(|c| rel(&root, &c.path)).collect();
    actual.sort();
    assert_eq!(actual, expected, "{name}: changed files; notes: {:#?}", m.notes);
    for c in &m.changes {
        let want = std::fs::read_to_string(expected_dir.join(rel(&root, &c.path))).unwrap().replace("\r\n", "\n");
        assert_eq!(c.after, want, "{name}: {}\n{}", rel(&root, &c.path), c.unified_diff("x"));
    }
    assert_eq!(m.notes.len(), notes.len(), "{name}: notes {:#?}", m.notes);
    for n in notes {
        assert!(m.notes.iter().any(|note| note.contains(n)), "{name}: no note containing {n:?} in {:#?}", m.notes);
    }
    idempotent(name);
    m
}

/// After writing the plan into a copy, a second plan changes nothing.
fn idempotent(name: &str) {
    let copy = Path::new(env!("CARGO_TARGET_TMPDIR")).join("migrate").join(name);
    let _ = std::fs::remove_dir_all(&copy);
    let src = fixture(name);
    let mut files = Vec::new();
    files_under(&src, &mut files);
    for f in files {
        let to = copy.join(rel(&src, &f));
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(&f, to).unwrap();
    }
    for c in plan(&copy, VERSION).changes {
        c.write().unwrap();
    }
    let again = plan(&copy, VERSION);
    assert!(again.changes.is_empty(), "{name}: second run changes {:#?}", again.changes);
}

#[test]
fn ktfmt_gradle_plugins_block() {
    check("migrate-ktfmt-gradle", &[]);
}

#[test]
fn ktlint_gradle_groovy_with_unsupported_version() {
    check("migrate-ktlint-groovy", &["build.gradle: ktlint { version = \"1.4.0\" }"]);
}

#[test]
fn catalog_version_ref() {
    check("ktfmt-catalog", &[]);
}

#[test]
fn catalog_alias_applied_in_subprojects() {
    check("ktlint-multi", &[]);
}

#[test]
fn convention_plugin_literal_dependency() {
    check("migrate-convention", &[]);
}

#[test]
fn convention_plugin_catalog_dependency() {
    check("migrate-convention-catalog", &[]);
}

#[test]
fn spotless_gradle_steps() {
    check("migrate-spotless-kts", &[]);
    check("spotless-root", &["build.gradle.kts: Spotless ktlint(): customRuleSets"]);
    check("groovy-legacy", &["applies ktfmt-gradle/ktlint-gradle/kotlinter by id"]);
}

#[test]
fn spotless_in_convention_plugin_is_a_note() {
    check(
        "migrate-spotless-convention",
        &["spotless-conventions.gradle.kts: Spotless ktfmt()/ktlint() in a convention plugin"],
    );
    check("convention-binary", &["my/Spotless.kt: Spotless ktfmt()/ktlint() in a convention plugin"]);
}

#[test]
fn unresolvable_versions_are_notes() {
    check("migrate-version-expr", &["isn't a string literal", "Spotless 6.25.0"]);
}

#[test]
fn maven() {
    check("maven-gantsign", &[]);
    check("migrate-maven-spotless", &["Spotless <ktlint> version 1.4.1"]);
    check("maven-spotless", &[]);
}

#[test]
fn kotlinter_plugins_block_and_catalog() {
    check("kotlinter-groovy", &[]);
    check("migrate-kotlinter-catalog", &[]);
}

#[test]
fn kotlinter_settings_the_drop_in_differs_on_are_notes() {
    check(
        "migrate-kotlinter-notes",
        &[
            "build.gradle.kts: kotlinter 4.4.1: the drop-in has kotlinter 5's DSL",
            "build.gradle.kts: kotlinter { ktlintVersion = \"1.5.0\" }: the kotlinter drop-in runs only",
            "build.gradle.kts: ktlint(\"some.group:custom-rules:1.0\"): only compose-rules runs natively",
        ],
    );
}

#[test]
fn no_drop_in_setups_are_notes() {
    check("ktlint-javaexec", &["build.gradle.kts: ktlint runs from its jar"]);
    check("maven-antrun", &["pom.xml: ktlint runs from its jar"]);
}

#[test]
fn migrated_and_unrelated_builds_are_left_alone() {
    for name in ["ktrs-step", "kotlinter-ktrs", "maven-ktrs", "convention-script", "none"] {
        check(name, &[]);
    }
}

/// What migration adds is what README "Integrations" tells users to write.
#[test]
fn added_lines_match_the_readme() {
    let readme = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md")).unwrap();
    let marker = "id(\"io.github.hexay.ktrs\") version \"";
    let start = readme.find(marker).expect("README ktfmt-gradle snippet") + marker.len();
    let version = &readme[start..start + readme[start..].find('"').unwrap()];
    let readme_lines: Vec<&str> = readme
        .lines()
        .map(|l| l.split(" // was:").next().unwrap().split(" <!-- was:").next().unwrap().trim())
        .collect();
    for name in ["migrate-ktfmt-gradle", "migrate-spotless-kts", "maven-gantsign", "migrate-maven-spotless"] {
        let m = plan(&fixture(name), version);
        for c in &m.changes {
            for line in c.unified_diff("x").lines().filter(|l| l.starts_with('+') && !l.starts_with("+++")) {
                let line = line[1..].trim();
                assert!(line.is_empty() || readme_lines.contains(&line), "{name}: `{line}` isn't in the README");
            }
        }
    }
}
