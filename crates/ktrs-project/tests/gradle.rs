mod common;

use common::*;
use ktrs_project::FormatTool::{Ktfmt, Ktlint};
use ktrs_project::KtfmtStyle::{Google, Kotlinlang};
use ktrs_project::KtlintVersion::{V1_8, V2_0};

#[test]
fn ktfmt_gradle_through_a_catalog_alias() {
    let c = detect_fixture("ktfmt-catalog/src/main/kotlin/Main.kt");
    assert_eq!(c.root, fixture("ktfmt-catalog"));
    let expected = ktfmt_settings(Kotlinlang, |k| {
        k.max_width = Some(120);
        k.remove_unused_imports = Some(false);
        k.manage_trailing_commas = Some(false);
    });
    assert_eq!(c.format, Some(Ktfmt(expected)), "{:#?}", c.notes);
    assert_eq!(c.ktlint, None);
    assert!(has_note(&c, "build.gradle.kts: plugin com.ncorti.ktfmt.gradle"), "{:#?}", c.notes);
}

#[test]
fn ktlint_gradle_multi_module() {
    let app = detect_fixture("ktlint-multi/app/src/main/kotlin/Main.kt");
    assert_eq!(app.root, fixture("ktlint-multi"));
    let expected = ktlint_config(V2_0, |k| {
        k.android = Some(true);
        k.experimental = Some(true);
        k.editorconfig_overrides = pairs(&[("max_line_length", "120"), ("ktlint_code_style", "intellij_idea")]);
        k.rule_sets = strings(&["io.nlopez.compose.rules:ktlint:0.4.22"]);
    });
    assert_eq!(app.ktlint, Some(expected), "{:#?}", app.notes);
    assert_eq!(app.format, Some(Ktlint));

    let lib = detect_fixture("ktlint-multi/lib/src/main/kotlin/Main.kt");
    let expected = ktlint_config(V2_0, |k| {
        k.android = Some(true);
        k.editorconfig_overrides = pairs(&[("max_line_length", "120")]);
    });
    assert_eq!(lib.ktlint, Some(expected), "{:#?}", lib.notes);

    // `subprojects {}` doesn't reach the root project, and the root's plugin is `apply false`.
    let root = detect_fixture("ktlint-multi/build.gradle.kts");
    assert_eq!((root.format, root.ktlint), (None, None), "{:#?}", root.notes);
}

#[test]
fn spotless_root_sections() {
    let core = detect_fixture("spotless-root/core/src/main/kotlin/Main.kt");
    let google = ktfmt_settings(Google, |k| {
        k.max_width = Some(80);
        k.block_indent = Some(2);
    });
    assert_eq!(core.format, Some(Ktfmt(google.clone())), "{:#?}", core.notes);
    assert_eq!(core.ktlint, None);

    let script = detect_fixture("spotless-root/build.gradle.kts");
    let expected = ktlint_config(V1_8, |k| {
        k.editorconfig_overrides = pairs(&[("indent_size", "2")]);
        k.rule_sets = strings(&["io.nlopez.compose.rules:ktlint:0.4.22"]);
    });
    assert_eq!(script.ktlint, Some(expected), "{:#?}", script.notes);
    assert_eq!(script.format, Some(Ktlint));

    let root_source = detect_fixture("spotless-root/src/main/kotlin/Root.kt");
    assert_eq!(root_source.format, Some(Ktfmt(google)));
}

#[test]
fn kotlinter_groovy() {
    let c = detect_fixture("kotlinter-groovy/src/main/kotlin/Main.kt");
    let expected = ktlint_config(V1_8, |k| k.rule_sets = strings(&["io.nlopez.compose.rules:ktlint:0.4.22"]));
    assert_eq!(c.ktlint, Some(expected), "{:#?}", c.notes);
    assert_eq!(c.format, Some(Ktlint));
    assert!(has_note(&c, "kotlinter: no ktlint version set"), "{:#?}", c.notes);
}

#[test]
fn kotlinter_drop_in_id() {
    let c = detect_fixture("kotlinter-ktrs/src/main/kotlin/Main.kt");
    let expected = ktlint_config(V2_0, |k| k.rule_sets = strings(&["io.nlopez.compose.rules:ktlint:0.6.7"]));
    assert_eq!(c.ktlint, Some(expected), "{:#?}", c.notes);
    assert_eq!(c.format, Some(Ktlint));
    assert!(has_note(&c, "build.gradle.kts: plugin io.github.hexay.ktrs.kotlinter"), "{:#?}", c.notes);
}

#[test]
fn groovy_legacy_apply_ext_and_properties() {
    let c = detect_fixture("groovy-legacy/src/main/kotlin/Main.kt");
    let expected = ktlint_config(V1_8, |k| {
        k.android = Some(true);
        k.editorconfig_overrides = pairs(&[("ktlint_code_style", "android_studio"), ("max_line_length", "100")]);
        k.rule_sets = strings(&["com.twitter.compose.rules:ktlint:0.0.26"]);
    });
    assert_eq!(c.ktlint, Some(expected), "{:#?}", c.notes);
    assert_eq!(c.format, Some(Ktfmt(ktfmt_settings(Kotlinlang, |k| k.max_width = Some(110)))));
}

#[test]
fn precompiled_convention_scripts() {
    let c = detect_fixture("convention-script/app/src/main/kotlin/Main.kt");
    assert_eq!(c.format, Some(Ktfmt(ktfmt_settings(Google, |k| k.max_width = Some(90)))), "{:#?}", c.notes);
    assert_eq!(c.ktlint, Some(ktlint(V2_0)));
    assert!(has_note(&c, "build-logic/src/main/kotlin/my.lint-conventions.gradle.kts: plugin"), "{:#?}", c.notes);
}

#[test]
fn binary_convention_plugin_with_helper_function() {
    let c = detect_fixture("convention-binary/app/src/main/kotlin/Main.kt");
    let expected = ktlint_config(V1_8, |k| k.editorconfig_overrides = pairs(&[("android", "true")]));
    assert_eq!(c.ktlint, Some(expected), "{:#?}", c.notes);
    assert_eq!(c.format, Some(Ktlint));
    assert!(has_note(&c, "ktlint 1.4.0 is not supported"), "{:#?}", c.notes);
}

#[test]
fn ktrs_spotless_steps() {
    let c = detect_fixture("ktrs-step/src/main/kotlin/Main.kt");
    assert_eq!(c.format, Some(Ktfmt(ktfmt_settings(Kotlinlang, |k| k.max_width = Some(120)))), "{:#?}", c.notes);
    let expected = ktlint_config(V2_0, |k| {
        k.editorconfig_overrides = pairs(&[("indent_size", "2")]);
        k.rule_sets = strings(&["file(rules/compose.jar)"]);
    });
    assert_eq!(c.ktlint, Some(expected));
}

#[test]
fn ktlint_cli_configuration() {
    let c = detect_fixture("ktlint-javaexec/src/main/kotlin/Main.kt");
    let expected = ktlint_config(V1_8, |k| k.rule_sets = strings(&["project(:rules)"]));
    assert_eq!(c.ktlint, Some(expected.clone()), "{:#?}", c.notes);
    assert_eq!(c.format, Some(Ktlint));
    // The root's task lints `src/**/*.kt`, so it covers the other modules too.
    let sub = detect_fixture("ktlint-javaexec/sub/src/main/kotlin/Main.kt");
    assert_eq!(sub.ktlint, Some(expected), "{:#?}", sub.notes);
}

#[test]
fn nothing_detected() {
    let c = detect_fixture("none/src/main/kotlin/Main.kt");
    assert_eq!((c.format, c.ktlint), (None, None), "{:#?}", c.notes);
    assert_eq!(c.root, fixture("none"));
}
