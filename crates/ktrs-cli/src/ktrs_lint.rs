//! `ktrs lint`: maps its flags onto the `ktlint` drop-in's options and runs the same command, so output,
//! reporters and exit codes are ktlint's. Paths are shown relative to the working directory.
//!
//! `-R` loads rule set JARs like the drop-in: a compose-rules release ktrs ports runs natively, any other JAR
//! with a rule set (or an `artifact=` reporter JAR) hands the run to the ktlint jar, with the equivalent ktlint
//! argv ([`ktlint_argv`]); its output is what `ktrs lint` prints anyway. Unlike the drop-in's, `-R` is
//! repeatable (and takes comma lists).

use ktrs_lint::rule_provider::rule_providers_in;

use crate::changed_since::{ChangedFiles, OPTION as CHANGED_SINCE};
use crate::github_annotations::REPORTER_ID;
use crate::ktlint::KtlintCli;
use crate::ktlint::args::KtlintArgs;
use crate::ktlint::command_line::Exit;
use crate::ktlint::jar_providers::jvm_only_jar;
use crate::ktlint::ktlint_jar::run_ktlint_jar;
use crate::ktlint::ktrs_only::KtrsLintOptions;
use crate::ktlint::logger::{Level, Logger};
use crate::ktlint::version::{KTLINT_VERSION_OPTION, exit_value, resolve_ktlint_version};

pub const HELP: &str = "\
Lint options (ktrs lint; the flags of the ktlint drop-in):
  -F, --format                      Fix what can be autocorrected, report the rest
  --reporter <id[,output=file]>     plain (default), plain?group_by_file, plain-summary, json,
                                      checkstyle, sarif, html, format, github (GitHub Actions
                                      annotations, `::error file=...`); repeatable
  --changed-since <ref>             Only files that differ from the merge base of <ref> and HEAD,
                                      uncommitted and untracked ones included (needs git)
  -R, --ruleset <jar[,jar]>         Also run the rules of a ktlint rule set JAR; repeatable. Other
                                      than compose-rules, runs ktlint's jar (needs `java`)
  --baseline <file>                 Ignore the violations recorded in <file> (written if missing)
  --editorconfig <file>             Defaults for properties no .editorconfig on a file's path sets
  --stdin-name <path>               Path of the stdin input (-), for .editorconfig and file name rules
  --limit <n>                       Report at most <n> violations
  --ktlint-version <1.8|2.0>        The ktlint release to match (default: ktrs_ktlint_version in
                                      .editorconfig, else 2.0)
  --list-rules                      Print the ids of the rules of that release (and of -R JARs)";

const LIST_RULES: &str = "--list-rules";

/// `Err` is a usage error message.
pub fn run(args: &[String]) -> Result<i32, String> {
    run_with(&KtlintCli::from_env(), args)
}

/// [`run`] in `cli`'s working directory and console.
pub fn run_with(cli: &KtlintCli, args: &[String]) -> Result<i32, String> {
    let ktlint_version = resolve_ktlint_version(args, &cli.working_dir.to_path_buf())?;
    let mut parsed = KtlintArgs { ktlint_version, ..parse_lint_args(args)? };
    let list_rules = args.iter().any(|a| a == LIST_RULES);
    if let Some(jar) = jvm_only_jar(&parsed, &cli.working_dir, &cli.user_home) {
        if list_rules {
            return Err(format!("{LIST_RULES} can not list the rules of '{jar}', a ktlint plugin JAR ktrs can not run natively"));
        }
        if let Some(option) = ktrs_only_option(&parsed) {
            return Err(format!("{option} is not available with '{jar}', a ktlint plugin JAR that runs on ktlint's jar"));
        }
        return Ok(run_ktlint_jar(&cli.jvm, ktlint_version, &ktlint_argv(&parsed), &cli.working_dir.to_path_buf(), &cli.console, &jar));
    }
    if list_rules {
        return Ok(print_rule_ids(cli, &parsed));
    }
    if let Some(reference) = &parsed.ktrs_lint.changed_since {
        parsed.ktrs_lint.changed_files = Some(ChangedFiles::since(reference, &cli.working_dir.to_path_buf())?);
    }
    ktrs_lint::engine::silence_caught_rule_panics();
    Ok(cli.run_lint(&parsed))
}

/// The option of this run that ktlint's jar does not have, if any.
fn ktrs_only_option(args: &KtlintArgs) -> Option<String> {
    let is_github = |reporter: &String| reporter.split([',', '?']).next() == Some(REPORTER_ID);
    if args.ktrs_lint.changed_since.is_some() {
        Some(CHANGED_SINCE.to_owned())
    } else {
        args.reporter_configurations.iter().any(is_github).then(|| format!("--reporter {REPORTER_ID}"))
    }
}

fn print_rule_ids(cli: &KtlintCli, args: &KtlintArgs) -> i32 {
    let logger = Logger::new(cli.console.clone(), Level::Error, args.ktlint_version);
    match cli.rule_providers(args, &logger) {
        Ok(providers) => {
            let ids: String = rule_providers_in(&providers, args.ktlint_version).iter().map(|p| format!("{}\n", p.rule_id().value())).collect();
            cli.console.out(&ids);
            0
        }
        Err(Exit::Code(code)) => exit_value(args.ktlint_version, code),
        Err(_) => 1,
    }
}

/// The options without `--ktlint-version`, which [`resolve_ktlint_version`] reads.
pub fn parse_lint_args(args: &[String]) -> Result<KtlintArgs, String> {
    let ktrs_lint = KtrsLintOptions { github_reporter: true, ..KtrsLintOptions::default() };
    let mut parsed = KtlintArgs { relative: true, ktrs_lint, ..KtlintArgs::default() };
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let (flag, inline_value) = match arg.split_once('=') {
            Some((flag, value)) if arg.starts_with("--") => (flag, Some(value.to_owned())),
            _ => (arg.as_str(), None),
        };
        let mut value = || inline_value.clone().or_else(|| args.next().cloned()).ok_or(format!("{flag} needs a value"));
        match flag {
            "-F" | "--format" => parsed.format = true,
            "--reporter" => parsed.reporter_configurations.push(value()?),
            "-R" | "--ruleset" => parsed.ruleset_jar_paths.extend(value()?.split(',').map(str::to_owned)),
            "--baseline" => parsed.baseline_path = value()?,
            "--editorconfig" => parsed.editor_config_path = Some(value()?),
            "--stdin-name" => parsed.stdin_path = Some(value()?),
            CHANGED_SINCE => parsed.ktrs_lint.changed_since = Some(value()?),
            KTLINT_VERSION_OPTION => drop(value()?),
            LIST_RULES => {}
            "--limit" => {
                let limit = value()?;
                parsed.limit = limit.parse().ok().filter(|n| *n > 0).ok_or(format!("--limit needs a positive number, not {limit}"))?;
            }
            "-" => parsed.stdin = true,
            _ if flag.starts_with('-') => return Err(format!("unknown option {arg}")),
            _ => parsed.arguments.push(arg.clone()),
        }
    }
    if parsed.stdin && !parsed.arguments.is_empty() {
        return Err("cannot read from stdin and files in the same run".to_owned());
    }
    if parsed.stdin_path.is_some() && !parsed.stdin {
        return Err("--stdin-name can only be used when reading from stdin (-)".to_owned());
    }
    if parsed.stdin && parsed.ktrs_lint.changed_since.is_some() {
        return Err(format!("{CHANGED_SINCE} can not be used when reading from stdin (-)"));
    }
    if !parsed.stdin && parsed.arguments.is_empty() {
        parsed.arguments.push(".".to_owned());
    }
    Ok(parsed)
}

/// The ktlint CLI argv of a `ktrs lint` run, for the jar hand-off. Arguments never start with `-` here (those
/// are options), so no `--` is needed; a leading `@` is escaped, as Clikt would read an argfile.
pub fn ktlint_argv(args: &KtlintArgs) -> Vec<String> {
    let mut argv = vec!["--relative".to_owned()];
    let mut option = |name: &str, value: &str| argv.push(format!("{name}={value}"));
    args.reporter_configurations.iter().for_each(|r| option("--reporter", r));
    if !args.ruleset_jar_paths.is_empty() {
        option("--ruleset", &args.ruleset_jar_paths.join(","));
    }
    if !args.baseline_path.is_empty() {
        option("--baseline", &args.baseline_path);
    }
    if let Some(editor_config) = &args.editor_config_path {
        option("--editorconfig", editor_config);
    }
    if args.limit != KtlintArgs::default().limit {
        option("--limit", &args.limit.to_string());
    }
    if let Some(stdin_path) = &args.stdin_path {
        option("--stdin-path", stdin_path);
    }
    if args.format {
        argv.push("--format".to_owned());
    }
    if args.stdin {
        argv.push("--stdin".to_owned());
    }
    argv.extend(args.arguments.iter().map(|a| if a.starts_with('@') { format!("@{a}") } else { a.clone() }));
    argv
}
