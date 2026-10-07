mod common;

use std::path::PathBuf;

use common::*;
use ktrs_project::FormatTool::{Ktfmt, Ktlint};
use ktrs_project::KtfmtStyle::{Google, Kotlinlang};
use ktrs_project::KtlintVersion::{V1_8, V2_0};
use ktrs_project::{FormatTool, detect, invalidate, is_build_file};

#[test]
fn gantsign_through_parent_plugin_management() {
    let c = detect_fixture("maven-gantsign/module/src/main/kotlin/Main.kt");
    assert_eq!(c.root, fixture("maven-gantsign"));
    let expected = ktlint_config(V1_8, |k| {
        k.android = Some(true);
        k.experimental = Some(true);
        k.rule_sets = strings(&["io.nlopez.compose.rules:ktlint:0.4.22"]);
    });
    assert_eq!(c.ktlint, Some(expected), "{:#?}", c.notes);
    assert_eq!(c.format, Some(Ktlint));
    assert!(has_note(&c, "module/pom.xml: ktlint-maven-plugin"), "{:#?}", c.notes);
}

#[test]
fn spotless_maven_ktfmt_and_ktlint() {
    let c = detect_fixture("maven-spotless/src/main/kotlin/Main.kt");
    let expected = ktfmt_settings(Kotlinlang, |k| {
        k.max_width = Some(100);
        k.manage_trailing_commas = Some(true);
    });
    assert_eq!(c.format, Some(Ktfmt(expected)), "{:#?}", c.notes);
    let expected = ktlint_config(V2_0, |k| {
        k.editorconfig_overrides = pairs(&[("ktlint_code_style", "intellij_idea")]);
        k.rule_sets = strings(&["io.nlopez.compose.rules:ktlint:0.4.22"]);
    });
    assert_eq!(c.ktlint, Some(expected));
    assert!(has_note(&c, "implementation io.github.hexay.ktrs.spotless.maven.KtrsKtfmt"), "{:#?}", c.notes);
}

#[test]
fn antrun_ktlint_cli() {
    let c = detect_fixture("maven-antrun/src/main/kotlin/Main.kt");
    let expected = ktlint_config(V1_8, |k| k.rule_sets = strings(&["io.nlopez.compose.rules:ktlint:0.4.22"]));
    assert_eq!(c.ktlint, Some(expected), "{:#?}", c.notes);
    assert!(has_note(&c, "ktlint 1.4.1 is not supported; using 1.8"), "{:#?}", c.notes);
}

#[test]
fn ktrs_maven_plugin_version() {
    let c = detect_fixture("maven-ktrs");
    assert_eq!(c.ktlint, Some(ktlint(V2_0)), "{:#?}", c.notes);
    assert_eq!(c.format, Some(Ktlint));
}

#[test]
fn build_files() {
    for f in ["a/build.gradle.kts", "settings.gradle", "pom.xml", "gradle/libs.versions.toml", "x.gradle"] {
        assert!(is_build_file(&PathBuf::from(f)), "{f}");
    }
    assert!(is_build_file(&PathBuf::from("build-logic/src/main/kotlin/Conv.kt")));
    assert!(!is_build_file(&PathBuf::from("src/main/kotlin/Main.kt")));
    assert!(!is_build_file(&PathBuf::from("app/src/test.kts")));
}

fn write(path: &PathBuf, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn style(c: &ktrs_project::ProjectConfig) -> Option<ktrs_project::KtfmtStyle> {
    match &c.format {
        Some(FormatTool::Ktfmt(k)) => Some(k.style),
        _ => None,
    }
}

#[test]
fn caches_until_invalidated() {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!("ktrs-project-cache-{}-{nanos}", std::process::id()));
    let build = root.join("build.gradle.kts");
    let source = root.join("app/src/Main.kt");
    write(&root.join("settings.gradle.kts"), "include(\":app\")\n");
    write(&build, "plugins { id(\"com.ncorti.ktfmt.gradle\") }\nktfmt { googleStyle() }\n");
    write(&source, "fun main() {}\n");
    assert_eq!(style(&detect(&source)), Some(Google));

    write(&build, "plugins { id(\"com.ncorti.ktfmt.gradle\") }\nktfmt { kotlinLangStyle() }\n");
    assert_eq!(style(&detect(&source)), Some(Google), "cached");
    invalidate(&build);
    // `app` has no build file yet, so the root build is its module.
    assert_eq!(style(&detect(&source)), Some(Kotlinlang));

    // A new module build file under the root drops the root's entries.
    let app_build = root.join("app/build.gradle.kts");
    write(&app_build, "plugins { id(\"org.jlleitschuh.gradle.ktlint\") }\n");
    invalidate(&app_build);
    let c = detect(&source);
    assert_eq!(c.format, Some(Ktlint));
    assert_eq!(c.ktlint.map(|k| k.version), Some(V1_8));
    let _ = std::fs::remove_dir_all(&root);
}
