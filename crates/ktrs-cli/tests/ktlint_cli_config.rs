//! Ports of ktlint-cli's `BaselineCLITest`, `EditorConfigDefaultsLoaderCLITest` and `RuleSetsLoaderCLITest`.

mod ktlint_support;

use ktlint_support::{Project, contains_line};

// Upstream's baseline files violate no-empty-class-body and no-single-line-block-comment (not ported yet):
// here no-semi (in the baseline) and no-wildcard-imports (not in it).
const BASELINE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<baseline version="1.0">
    <file name="TestBaselineExtraErrorFile.kt.test">
        <error line="3" column="12" source="standard:no-semi" />
    </file>
    <file name="TestBaselineFile.kt.test">
        <error line="1" column="10" source="standard:no-semi" />
    </file>
    <file name="some/path/to/TestBaselineExtraErrorFile.kt.test">
        <error line="3" column="12" source="standard:no-semi" />
    </file>
    <file name="some/path/to/TestBaselineFile.kt.test">
        <error line="1" column="10" source="standard:no-semi" />
    </file>
</baseline>
"#;
const BASELINE_FILE: &str = "val x = 1;\n";
const BASELINE_EXTRA_ERROR_FILE: &str = "import foo.*\n\nval x = foo;\n";

fn baseline_project() -> Project {
    Project::new(
        "baseline",
        &[
            ("test-baseline.xml", BASELINE_XML),
            ("config/test-baseline.xml", BASELINE_XML),
            ("TestBaselineFile.kt.test", BASELINE_FILE),
            ("TestBaselineExtraErrorFile.kt.test", BASELINE_EXTRA_ERROR_FILE),
            ("some/path/to/TestBaselineFile.kt.test", BASELINE_FILE),
            ("some/path/to/TestBaselineExtraErrorFile.kt.test", BASELINE_EXTRA_ERROR_FILE),
        ],
    )
}

const FILES: [&str; 2] = ["TestBaselineFile.kt.test", "some/path/to/TestBaselineFile.kt.test"];

#[test]
fn baseline_ignored_then_find_the_errors() {
    let run = baseline_project().run(&FILES);
    run.assert_error_exit_code();
    assert_line!(run.out, ".*/baseline/TestBaselineFile.kt.test:1:10: Unnecessary semicolon.*");
    assert_line!(run.out, ".*/baseline/some/path/to/TestBaselineFile.kt.test:1:10: Unnecessary semicolon.*");
}

fn assert_all_ignored(run: &ktlint_support::Run) {
    run.assert_normal_exit_code();
    assert_no_line!(run.out, ".*TestBaselineFile.kt.test:1:10: Unnecessary semicolon.*");
    assert_no_line!(run.out, ".*Format was not able to resolve all violations which \\(theoretically\\) can be autocorrected in file.*");
}

#[test]
fn baseline_in_the_root_of_the_working_directory() {
    let project = baseline_project();
    assert_all_ignored(&project.run(&["--baseline=test-baseline.xml", "--format", FILES[0], FILES[1]]));
    assert_eq!(project.read("TestBaselineFile.kt.test"), BASELINE_FILE);
}

#[test]
fn baseline_in_a_subdirectory_of_the_working_directory() {
    assert_all_ignored(&baseline_project().run(&["--baseline=config/test-baseline.xml", FILES[0], FILES[1]]));
}

#[test]
fn baseline_with_an_absolute_path() {
    let project = baseline_project();
    let baseline = format!("--baseline={}", project.dir().join("config/test-baseline.xml").display());
    assert_all_ignored(&project.run(&[&baseline, FILES[0], FILES[1]]));
}

#[test]
fn errors_not_in_the_baseline_are_reported() {
    let run = baseline_project().run(&[
        "--baseline=test-baseline.xml",
        "TestBaselineExtraErrorFile.kt.test",
        "some/path/to/TestBaselineExtraErrorFile.kt.test",
    ]);
    run.assert_error_exit_code();
    assert_line!(run.out, ".*/baseline/TestBaselineExtraErrorFile.kt.test:1:1: Wildcard import.*");
    assert_line!(run.out, ".*/baseline/some/path/to/TestBaselineExtraErrorFile.kt.test:1:1: Wildcard import.*");
    assert_no_line!(run.out, ".*Unnecessary semicolon.*");
}

fn editorconfig_path_project() -> Project {
    Project::new(
        "editorconfig-path",
        &[
            (
                "project/.editorconfig",
                "root = true\n\n[Foo*.{kt,kts}.test]\nmax_line_length = 30\n\n[src/**/example/*.kt.test]\nktlint_standard_no-wildcard-imports = disabled\n",
            ),
            ("project/.editorconfig-bar", "root = true\n\n[Bar*.{kt,kts}.test]\nmax_line_length = 20\n"),
            ("project/.editorconfig-default-max-line-length-on-tests-only", "root = true\n\n[*Test.{kt,kts}.test]\nmax_line_length = 25\n"),
            (
                "project/.editorconfig-disable-no-wildcard-imports-rule",
                "root = true\n\n[**/*-example/*.{kt,kts}.test]\nktlint_standard_no-wildcard-imports = disabled\n",
            ),
            ("project/editorconfig-alternative", "root = true\n\n[Bar*.{kt,kts}.test]\nmax_line_length = 20\n"),
            (
                "project/editorconfig-boolean-setting",
                "root = true\n\n[*.{kt,kts}.test]\nij_kotlin_allow_trailing_comma=true\nij_kotlin_allow_trailing_comma_on_call_site=true\n",
            ),
            ("project/src/main/kotlin/example/Bar.kts.test", "val bar = \"barbarbarbarbarbarbarbarbarbarbarbarbar\"\n"),
            ("project/src/main/kotlin/example/Foo.kt.test", "val foo = \"fooooooooooooooooooooooooooooooooooooooooo\"\n"),
            ("project/src/main/kotlin/example/Wildard1.kt.test", "import foo.bar.*\n\nfun wildcard1() {}\n"),
            ("project/src/main/kotlin/filename-example/Wildcard2.kt.test", "import foo.bar.*\n\nfun wildcard2() {}\n"),
            ("project/src/test/kotlin/example/BarTest.kts.test", "val barTest = \"barbarbarbarbarbarbarbarbarbarbarbarbar\"\n"),
            ("project/src/test/kotlin/example/FooTest.kt.test", "val fooTest = \"fooooooooooooooooooooooooooooooooooooooooo\"\n"),
        ],
    )
}

fn editorconfig_arg(project: &Project, file: &str) -> String {
    format!("--editorconfig={}/project/{file}", project.dir().display())
}

#[test]
fn without_default_editorconfig_only_the_editorconfig_files_on_the_path_are_used() {
    let run = editorconfig_path_project().run(&["**/*.test"]);
    run.assert_error_exit_code();
    assert_line!(run.out, ".*Foo.*Exceeded max line length \\(30\\).*");
    assert_line!(run.out, ".*Wildcard2.*Wildcard import.*");
    assert_no_line!(run.out, ".*Bar.*Exceeded max line length.*");
    assert_no_line!(run.out, ".*Wildard1.*Wildcard import.*");
}

#[test]
fn default_editorconfig_is_used_when_the_files_on_the_path_do_not_set_the_property() {
    for path in [".editorconfig-bar", "editorconfig-alternative", "../project/editorconfig-alternative"] {
        let project = editorconfig_path_project();
        let run = project.run(&["**/*.test", &editorconfig_arg(&project, path)]);
        run.assert_error_exit_code();
        assert_line!(run.out, ".*FooTest.*Exceeded max line length \\(30\\).*");
        assert_line!(run.out, ".*Foo.*Exceeded max line length \\(30\\).*");
        assert_line!(run.out, ".*BarTest.*Exceeded max line length \\(20\\).*");
        assert_line!(run.out, ".*Bar.*Exceeded max line length \\(20\\).*");
    }
}

#[test]
fn default_editorconfig_max_line_length_for_test_files_only() {
    let project = editorconfig_path_project();
    let run = project.run(&["**/*.test", &editorconfig_arg(&project, ".editorconfig-default-max-line-length-on-tests-only")]);
    run.assert_error_exit_code();
    assert_line!(run.out, ".*FooTest.*Exceeded max line length \\(30\\).*");
    assert_line!(run.out, ".*Foo.*Exceeded max line length \\(30\\).*");
    assert_line!(run.out, ".*BarTest.*Exceeded max line length \\(25\\).*");
}

#[test]
fn default_editorconfig_disables_no_wildcard_imports_for_all_example_files() {
    let project = editorconfig_path_project();
    let run = project.run(&["**/*.test", &editorconfig_arg(&project, ".editorconfig-disable-no-wildcard-imports-rule")]);
    run.assert_error_exit_code();
    assert_no_line!(run.out, ".*Wildard1.*Wildcard import.*");
    assert_no_line!(run.out, ".*Wildcard2.*Wildcard import.*");
}

#[test]
fn issue_1627_default_editorconfig_with_a_boolean_setting() {
    let project = editorconfig_path_project();
    let run = project.run(&["**/*.test", &editorconfig_arg(&project, "editorconfig-boolean-setting")]);
    run.assert_error_exit_code();
    assert!(!contains_line(&run.err, "ClassCastException"), "{run:?}");
}

/// A stand-in for a JAR declaring `service` (the central directory stores entry names uncompressed).
fn fake_jar(service: &str) -> Vec<u8> {
    let mut bytes = b"PK\x03\x04".to_vec();
    bytes.extend_from_slice(format!("META-INF/services/{service}").as_bytes());
    bytes
}

#[test]
fn custom_jar_without_a_rule_set_provider_then_error_and_exit() {
    let project = Project::new("custom-ruleset", &[]);
    project.write_bytes("custom-ruleset/ktlint-cli-reporter-html.jar", &fake_jar("io.github.ktlint.core.cli.reporter.core.api.ReporterProviderV2"));
    let jar = project.dir().join("custom-ruleset/ktlint-cli-reporter-html.jar").display().to_string();
    let run = project.run(&["-R", &jar, "**/*.test"]);
    assert_eq!(run.exit_code, 6, "{run:?}");
    assert_line!(
        format!("{}{}", run.out, run.err),
        ".*ERROR.* JAR file '.*custom-ruleset/ktlint-cli-reporter-html.jar' is missing a class implementing interface 'io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider'.*"
    );
}

#[test]
fn custom_jar_with_the_deprecated_rule_set_provider_v3_can_not_be_loaded() {
    // Deviation: upstream loads it (with a deprecation warning); JVM rule sets can't run here.
    let project = Project::new("custom-ruleset", &[]);
    let name = "custom-ruleset/ktlint-ruleset-with-deprecated-ruleset-provider.jar";
    project.write_bytes(name, &fake_jar("com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3"));
    let jar = project.dir().join(name).display().to_string();
    let run = project.run(&["-R", &jar, "**/*.test"]);
    assert_eq!(run.exit_code, 6, "{run:?}");
    assert_line!(run.out, ".*ERROR.* JAR file '.*ktlint-ruleset-with-deprecated-ruleset-provider.jar' .*can not load JVM code");
}
