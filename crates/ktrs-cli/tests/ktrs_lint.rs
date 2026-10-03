//! `ktrs lint`'s rule set JARs (`-R`): native compose-rules, the ktlint jar hand-off, and the drop-in's errors.

mod ktlint_support;

use std::path::Path;

use ktlint_support::{Project, Run};
use ktrs_cli::ktlint::args::KtlintArgs;
use ktrs_cli::ktrs_lint::{ktlint_argv, parse_lint_args, run_with};

/// Usage errors (`Err`) print to stderr and exit 2 here.
fn lint(project: &Project, args: &[&str]) -> Run {
    project.run_cli(args, b"", |cli, args| {
        run_with(cli, args).unwrap_or_else(|message| {
            cli.console.err(&message);
            2
        })
    })
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| a.to_string()).collect()
}

/// A stand-in for a JAR declaring `service` (the central directory stores entry names uncompressed).
fn fake_jar(service: &str) -> Vec<u8> {
    let mut bytes = b"PK\x03\x04".to_vec();
    bytes.extend_from_slice(format!("META-INF/services/{service}").as_bytes());
    bytes
}

const RULE_SET_V2_PROVIDER: &str = "io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider";

fn project_with_jar(service: &str) -> Project {
    let project = Project::new("ktrs-lint", &[("A.kt", "val a = 1\n")]);
    project.write_bytes("custom.jar", &fake_jar(service));
    project
}

#[test]
fn ruleset_is_repeatable_and_takes_comma_lists() {
    let parsed = parse_lint_args(&strings(&["-R", "a.jar,b.jar", "--ruleset=c.jar", "--ruleset", "d.jar", "src"])).unwrap();
    assert_eq!(parsed.ruleset_jar_paths, ["a.jar", "b.jar", "c.jar", "d.jar"]);
    assert_eq!(parsed.arguments, ["src"]);
    assert!(parse_lint_args(&strings(&["-R"])).is_err());
}

#[test]
fn jvm_rule_set_jar_hands_the_run_to_the_ktlint_jar() {
    let project = project_with_jar(RULE_SET_V2_PROVIDER);
    let run = lint(&project, &["-R", "custom.jar", "A.kt"]);
    assert_eq!(run.exit_code, 1, "{run:?}");
    assert_line!(run.err, "ktrs: 'custom.jar' is a ktlint plugin JAR .* ktlint 2.0.0-ALPHA-4 on the JVM, but no `java` was found.*");
    let project = project_with_jar("com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3");
    let run = lint(&project, &["-R", "custom.jar", "--ktlint-version=1.8", "A.kt"]);
    assert_eq!(run.exit_code, 1, "{run:?}");
    assert_eq!(run.out, "", "{run:?}");
    assert_line!(run.err, "ktrs: 'custom.jar' is a ktlint plugin JAR .* ktlint 1.8.0 on the JVM, but no `java` was found.*");
}

#[test]
fn reporter_artifact_jar_hands_the_run_to_the_ktlint_jar() {
    let project = project_with_jar("io.github.ktlint.core.cli.reporter.core.api.ReporterProviderV2");
    let run = lint(&project, &["--reporter=csv,artifact=custom.jar", "A.kt"]);
    assert_eq!(run.exit_code, 1, "{run:?}");
    assert_line!(run.err, "ktrs: 'custom.jar' is a ktlint plugin JAR .*no `java` was found.*");
}

#[test]
fn missing_jar_is_ktlints_file_not_found() {
    let project = Project::new("ktrs-lint", &[("A.kt", "val a = 1\n")]);
    let run = lint(&project, &["-R", "nothere.jar", "A.kt"]);
    assert_eq!(run.exit_code, 5, "{run:?}");
    assert_line!(format!("{}{}", run.out, run.err), ".*ERROR.* File 'nothere.jar' does not exist");
}

#[test]
fn jar_without_a_rule_set_provider_is_ktlints_invalid_ruleset_jar() {
    let project = project_with_jar("io.github.ktlint.core.cli.reporter.core.api.ReporterProviderV2");
    let run = lint(&project, &["-R", "custom.jar", "A.kt"]);
    assert_eq!(run.exit_code, 6, "{run:?}");
    assert_line!(format!("{}{}", run.out, run.err), ".*JAR file '.*custom.jar' is missing a class implementing interface '.*RuleSetV2Provider'.*");
}

#[test]
fn list_rules_names_the_standard_rules_and_refuses_jvm_jars() {
    let project = project_with_jar(RULE_SET_V2_PROVIDER);
    let run = lint(&project, &["--list-rules"]);
    assert_eq!(run.exit_code, 0, "{run:?}");
    assert_line!(run.out, "standard:no-semi");
    let run = lint(&project, &["--list-rules", "-R", "custom.jar"]);
    assert_eq!(run.exit_code, 2, "{run:?}");
    assert_eq!(run.err, "--list-rules can not list the rules of 'custom.jar', a ktlint plugin JAR ktrs can not run natively");
}

/// Needs `tools/sync-compose-rules.sh` (the JAR is not in git); skipped without it.
#[test]
fn pinned_compose_rules_jar_runs_natively() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/compose-rules/lib");
    let jar = lib.join(format!("ktlint-compose-{}-all.jar", ktrs_compose::COMPOSE_RULES_VERSION));
    if !jar.is_file() {
        return;
    }
    let project = Project::new("ktrs-lint", &[("A.kt", "val a = 1\n")]);
    let jar = jar.display().to_string();
    let run = lint(&project, &["--list-rules", "-R", &jar]);
    assert_eq!(run.exit_code, 0, "{run:?}");
    assert_line!(run.out, "compose:.*");
    assert_eq!(lint(&project, &["-R", &jar, "A.kt"]).exit_code, 0);
}

#[test]
fn hand_off_argv_is_the_equivalent_ktlint_command() {
    let parsed = KtlintArgs {
        format: true,
        reporter_configurations: strings(&["plain", "json,output=out.json"]),
        ruleset_jar_paths: strings(&["a.jar", "b.jar"]),
        baseline_path: "base.xml".to_owned(),
        editor_config_path: Some(".ec".to_owned()),
        limit: 3,
        arguments: strings(&["src", "@odd"]),
        ..parse_lint_args(&[]).unwrap()
    };
    assert_eq!(
        ktlint_argv(&parsed),
        [
            "--relative",
            "--reporter=plain",
            "--reporter=json,output=out.json",
            "--ruleset=a.jar,b.jar",
            "--baseline=base.xml",
            "--editorconfig=.ec",
            "--limit=3",
            "--format",
            "src",
            "@@odd"
        ]
    );
    let stdin = parse_lint_args(&strings(&["-", "--stdin-name", "x/A.kt"])).unwrap();
    assert_eq!(ktlint_argv(&stdin), ["--relative", "--stdin-path=x/A.kt", "--stdin"]);
}
