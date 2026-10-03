mod common;

use std::path::{Path, PathBuf};

use common::{TempDir, session, write_text};

/// A directory whose `.editorconfig` is the root one (so the machine's don't apply), with `A.kt` in it.
fn project(tag: &str, editorconfig: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new(tag);
    write_text(&dir.path().join(".editorconfig"), &format!("root = true\n{editorconfig}"));
    let file = dir.path().join("A.kt");
    (dir, file)
}

fn ktlint(file: &Path, header: &str, code: &str) -> String {
    let (exit, responses) = session(&[&format!("tool=ktlint\npath={}\n{header}\n{code}", file.display())]);
    assert_eq!(exit, 0);
    responses[1].clone()
}

#[test]
fn formats_and_reports_what_could_not_be_autocorrected() {
    let (_dir, file) = project("serve-ktlint-format", "");
    assert_eq!(ktlint(&file, "", "fun f() {\n  val x = 1\n}\n"), "status=ok\nchanged=true\n\nfun f() {\n    val x = 1\n}\n");
    assert_eq!(
        ktlint(&file, "", "import a.*\n\nfun  f() = a()\n"),
        "status=ok\nchanged=true\nviolation=1\t1\tstandard:no-wildcard-imports\tWildcard import\n\nimport a.*\n\nfun f() = a()\n"
    );
}

#[test]
fn file_name_rules_see_the_path() {
    let (_dir, file) = project("serve-ktlint-empty", "");
    assert_eq!(
        ktlint(&file, "", ""),
        "status=ok\nchanged=false\nviolation=1\t1\tstandard:no-empty-file\tFile 'A.kt' should not be empty\n\n"
    );
}

#[test]
fn overrides_win_over_editorconfig_and_default_to_intellij_idea() {
    let (_dir, file) = project("serve-ktlint-override", "[*.kt]\nindent_size = 2\n");
    let code = "fun f() {\n    val x = 1\n}\n";
    assert_eq!(ktlint(&file, "", code), "status=ok\nchanged=true\n\nfun f() {\n  val x = 1\n}\n");
    let three = "editorconfig-override=indent_size=3\neditorconfig-override=unknown_property=1\n";
    assert_eq!(ktlint(&file, three, code), "status=ok\nchanged=true\n\nfun f() {\n   val x = 1\n}\n");
    let disabled = "editorconfig-override=ktlint_standard_no-wildcard-imports=disabled\n";
    assert_eq!(ktlint(&file, disabled, "import a.*\n\nval x = a()\n"), "status=ok\nchanged=false\n\nimport a.*\n\nval x = a()\n");
    // ktlint_official adds a trailing comma to a multiline parameter list; any override switches to intellij_idea.
    let multiline = "fun f(\n    a: Int,\n    b: Int\n) = a + b\n";
    let (_dir2, file2) = project("serve-ktlint-official", "");
    assert!(ktlint(&file2, "", multiline).contains("b: Int,\n"));
    assert_eq!(ktlint(&file2, "editorconfig-override=max_line_length=200\n", multiline), "status=ok\nchanged=true\n\nfun f(a: Int, b: Int) = a + b\n");
}

#[test]
fn a_code_style_in_the_defaults_file_is_kept() {
    let (dir, file) = project("serve-ktlint-defaults", "");
    let defaults = dir.path().join("defaults.editorconfig");
    write_text(&defaults, "[*.{kt,kts}]\nktlint_code_style = ktlint_official\n");
    let multiline = "fun f(\n    a: Int,\n    b: Int\n) = a + b\n";
    let header = format!("editorconfig-defaults={}\neditorconfig-override=max_line_length=200\n", defaults.display());
    assert!(ktlint(&file, &header, multiline).contains("b: Int,\n"));
}

#[test]
fn ktlint_versions() {
    let (_dir, file) = project("serve-ktlint-versions", "");
    let code = "fun f() {\n  val x = 1\n}\n";
    let expected = "status=ok\nchanged=true\n\nfun f() {\n    val x = 1\n}\n";
    assert_eq!(ktlint(&file, "ktlint-version=1.8\n", code), expected);
    assert_eq!(ktlint(&file, "ktlint-version=2.0\n", code), expected);
    assert_eq!(ktlint(&file, "ktlint-version=1.7\n", code), "status=error\n\nktlint-version must be 1.8 or 2.0, got '1.7'");
}

#[test]
fn parse_errors_are_ktlints_exception_message() {
    let (_dir, file) = project("serve-ktlint-parse", "");
    let response = ktlint(&file, "", "fun f( {\n");
    assert!(response.starts_with("status=error\n\n1:"), "{response}");
}

#[test]
fn bad_requests_and_explicit_ktfmt() {
    let (_, responses) = session(&["tool=ktlint\nbogus=1\n\n", "tool=other\n\n", "tool=ktfmt\n\nval  x = 1\n"]);
    assert_eq!(responses[1], "status=error\n\nunknown request key 'bogus'");
    assert_eq!(responses[2], "status=error\n\ntool must be ktfmt or ktlint, got 'other'");
    assert_eq!(responses[3], "status=ok\nchanged=true\n\nval x = 1\n");
}

#[test]
fn only_the_compose_rules_jar_runs_natively() {
    let (dir, file) = project("serve-ktlint-ruleset", "");
    let other = dir.path().join("other.jar");
    write_text(&other, "not a jar");
    let response = ktlint(&file, &format!("ruleset={}\n", other.display()), "val x = 1\n");
    assert!(response.starts_with("status=error\n\nktrs runs no rule set JAR natively but compose-rules"), "{response}");
    assert!(response.contains("keep `ktlint()` for this rule set"), "{response}");
    let missing = ktlint(&file, &format!("ruleset={}\n", dir.path().join("missing.jar").display()), "val x = 1\n");
    assert!(missing.contains("does not exist"), "{missing}");

    let jar = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../tools/compose-rules/lib/ktlint-compose-{}-all.jar", ktrs_compose::COMPOSE_RULES_VERSION));
    if jar.is_file() {
        let code = "@Composable\nfun MyComposable() {\n    Text(\"x\")\n}\n";
        assert!(!ktlint(&file, "", code).contains(":compose:"));
        let response = ktlint(&file, &format!("ruleset={}\n", jar.display()), code);
        assert!(response.contains("violation=2\t5\tcompose:"), "{response}");
    }
}
