//! The `ktlint` drop-in in ktlint 1.8 mode (`--ktlint-version=1.8` or `ktrs_ktlint_version = 1.8`), against the
//! 1.8.0 jar's behaviour (research/26-ktlint-18-mode.md). The tree-wide comparison: tools/ktlint-oracle/cli-diff.sh.

#[path = "ktlint_support/mod.rs"]
mod ktlint_support;

use ktlint_support::Project;
use ktrs_cli::ktlint::args::HELP_MAIN;
use ktrs_cli::ktlint::console::LINE_SEPARATOR;
use ktrs_cli::ktlint::version::resolve_ktlint_version;
use ktrs_lint::editorconfig::KtlintVersion;

const V18: &str = "--ktlint-version=1.8";

fn project(files: &[(&str, &str)]) -> Project {
    Project::new("ktlint18", files)
}

fn tokens(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| a.to_string()).collect()
}

#[test]
fn the_version_comes_from_the_flag_else_the_working_directory_editorconfig() {
    let p = project(&[(".editorconfig", "root = true\n[*.{kt,kts}]\nktrs_ktlint_version = 1.8\n")]);
    assert_eq!(resolve_ktlint_version(&tokens(&[]), p.dir()), Ok(KtlintVersion::V1_8));
    assert_eq!(resolve_ktlint_version(&tokens(&["--ktlint-version", "2.0"]), p.dir()), Ok(KtlintVersion::V2_0));
    assert_eq!(resolve_ktlint_version(&tokens(&["--ktlint-version=2.0", "--", "--ktlint-version=1.8"]), p.dir()), Ok(KtlintVersion::V2_0));
    assert_eq!(resolve_ktlint_version(&tokens(&[]), project(&[]).dir()), Ok(KtlintVersion::V2_0));
    assert_eq!(p.run(&["--version"]).out, "ktlint version 1.8.0\n");
    assert_eq!(p.run(&["--ktlint-version=2.0", "--version"]).out, "ktlint version 2.0.0-ALPHA-4\n");
}

#[test]
fn an_invalid_version_is_a_usage_error() {
    let run = project(&[]).run(&["--ktlint-version=1.7", "src"]);
    assert_eq!(run.exit_code, 1);
    assert!(run.err.ends_with("Error: invalid value for --ktlint-version: invalid choice: 1.7. (choose from 1.8, 2.0)\n"), "{}", run.err);
}

#[test]
fn help_lists_the_deprecated_code_style_option_and_the_old_site() {
    let help = project(&[]).run(&[V18, "--help"]).out;
    assert!(help.contains("  (https://pinterest.github.io/ktlint/latest/).\n"), "{help}");
    assert!(help.contains(
        "  -v, --version                  Show the version and exit\n  \
         --code-style=(android_studio|intellij_idea|ktlint_official)\n                                 (deprecated)\n  --color "
    ));
    assert_eq!(help.lines().count(), HELP_MAIN.lines().count() + 2);
    assert_eq!(project(&[]).run(&["--help"]).out, HELP_MAIN);
}

#[test]
fn code_style_option_fails_with_the_migration_message() {
    let run = project(&[]).run(&[V18, "--code-style=ktlint_official", "src"]);
    assert_eq!((run.exit_code, run.err), (
        1,
        format!("Parameter '--code-style' is no longer valid. The code style should be defined as '.editorconfig' property 'ktlint_code_style='{LINE_SEPARATOR}")
    ));
    let run = project(&[]).run(&[V18, "--code-style", "A.kt"]);
    assert!(run.err.ends_with(
        "Error: invalid value for --code-style: invalid choice: A.kt. (choose from android_studio, intellij_idea, ktlint_official)\n"
    ));
    assert!(project(&[]).run(&["--code-style=ktlint_official"]).err.contains("Error: no such option --code-style"));
}

#[test]
fn disabled_rules_and_experimental_options_are_unknown_as_in_the_jar() {
    for option in ["--disabled_rules=x", "--experimental"] {
        let run = project(&[]).run(&[V18, option, "A.kt"]);
        let name = option.split('=').next().unwrap();
        assert!(run.err.ends_with(&format!("Error: no such option {name}\n")), "{}", run.err);
    }
    let run = project(&[]).run(&[V18, "--codestyle"]);
    assert!(run.err.ends_with("Error: no such option --codestyle. (Possible options: --code-style, --color-name)\n"), "{}", run.err);
}

#[test]
fn every_failure_exits_1_but_an_unparsable_format_result() {
    let p = project(&[("A.kt", "val a = 1\n")]);
    assert_eq!(p.run(&["-R", "nothere.jar", "A.kt"]).exit_code, 5);
    assert_eq!(p.run(&[V18, "-R", "nothere.jar", "A.kt"]).exit_code, 1);
    assert_eq!(p.run(&["--reporter=nope", "A.kt"]).exit_code, 7);
    assert_eq!(p.run(&[V18, "--reporter=nope", "A.kt"]).exit_code, 1);
}

#[test]
fn stdin_parse_failure_is_reported_as_a_row_with_exit_0() {
    for format in [false, true] {
        let args: &[&str] = if format { &[V18, "--stdin", "-F"] } else { &[V18, "--stdin"] };
        let run = project(&[]).run_with_stdin(args, b"fun a( = 1\n");
        assert_eq!(run.exit_code, 0);
        assert!(run.err.starts_with(&format!("<stdin>:1:7: Not a valid Kotlin file (1:7 expecting ')') (){LINE_SEPARATOR}")), "{}", run.err);
        assert!(run.out.contains(&format!(
            " ERROR com.pinterest.ktlint.cli.internal.KtlintCommandLine -- Can not parse input from <stdin> as Kotlin, due to error below:\n    \
             Not a valid Kotlin file (1:7 expecting ')'){LINE_SEPARATOR}"
        )), "{}", run.out);
        assert!(!run.out.contains("fun a("), "{}", run.out);
    }
    assert_eq!(project(&[]).run_with_stdin(&["--stdin"], b"fun a( = 1\n").exit_code, 3);
}

#[test]
fn stdin_format_output_goes_through_printf() {
    let run = project(&[]).run_with_stdin(&[V18, "--stdin", "-F"], b"val a = \"%d\"\n");
    assert_eq!(run.exit_code, 1);
    assert_eq!(run.err, format!("Exception in thread \"main\" java.util.MissingFormatArgumentException: Format specifier '%d'{LINE_SEPARATOR}"));
    let run = project(&[]).run_with_stdin(&[V18, "--stdin", "-F", "--log-level=none"], b"val a = \"100%%\"\n");
    assert_eq!((run.exit_code, run.out.as_str()), (0, "val a = \"100%\"\n"));
    let run = project(&[]).run_with_stdin(&["--stdin", "-F", "--log-level=none"], b"val a = \"%d\"\n");
    assert_eq!((run.exit_code, run.out.as_str()), (0, "val a = \"%d\"\n"));
}

#[test]
fn logs_carry_the_1_8_class_names() {
    let run = project(&[("A.kt", "val a = 1\n")]).run(&[V18]);
    assert!(run.out.contains(" INFO com.pinterest.ktlint.cli.internal.KtlintCommandLine -- Enable default patterns"), "{}", run.out);
}

#[test]
fn obsolete_disabled_rules_property_is_warned_about_per_file() {
    let editor_config = "root = true\n[*.{kt,kts}]\nktrs_ktlint_version = 1.8\ndisabled_rules = no-semi\n";
    let run = project(&[(".editorconfig", editor_config), ("A.kt", "val a = 1\n")]).run(&["A.kt"]);
    assert!(run.out.contains(
        " WARN com.pinterest.ktlint.rule.engine.internal.RuleExecutionContext -- Editorconfig property 'disabled_rules' is obsolete"
    ), "{}", run.out);
    let run = project(&[(".editorconfig", "root = true\n[*.kt]\ndisabled_rules = no-semi\n"), ("A.kt", "val a = 1\n")]).run(&["A.kt"]);
    assert!(!run.out.contains("obsolete"), "{}", run.out);
}

#[test]
fn context_receiver_list_wrapping_keeps_its_1_8_id() {
    let code = "context(aaaaaaaaaa: Aaaaaaaaaa, bbbbbbbbbb: Bbbbbbbbbbbb)\nfun f() = 1\n";
    let editor_config = "root = true\n[*.kt]\nmax_line_length = 50\n";
    let p = project(&[(".editorconfig", editor_config), ("A.kt", code)]);
    let run = p.run(&[V18, "--relative", "A.kt"]);
    assert!(run.out.contains("A.kt:1:9: Newline expected before context parameter as max line length is violated (standard:context-receiver-list-wrapping)"), "{}", run.out);
    // A new project: `.editorconfig` files are cached per process.
    let disabled = format!("{editor_config}ktlint_standard_context-receiver-list-wrapping = disabled\n");
    let run = project(&[(".editorconfig", &disabled), ("A.kt", code)]).run(&[V18, "--relative", "A.kt"]);
    assert!(!run.out.contains("context-"), "{}", run.out);
}
