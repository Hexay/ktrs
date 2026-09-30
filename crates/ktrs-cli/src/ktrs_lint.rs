//! `ktrs lint`: maps its flags onto the `ktlint` drop-in's options and runs the same command, so output,
//! reporters and exit codes are ktlint's. Paths are shown relative to the working directory.

use ktrs_lint::rules::standard_rule_providers;

use crate::ktlint::KtlintCli;
use crate::ktlint::args::KtlintArgs;

pub const HELP: &str = "\
Lint options (ktrs lint; the flags of the ktlint drop-in, 2.0.0-ALPHA-4):
  -F, --format                      Fix what can be autocorrected, report the rest
  --reporter <id[,output=file]>     plain (default), plain?group_by_file, plain-summary, json,
                                      checkstyle, sarif, html, format; repeatable
  --baseline <file>                 Ignore the violations recorded in <file> (written if missing)
  --editorconfig <file>             Defaults for properties no .editorconfig on a file's path sets
  --stdin-name <path>               Path of the stdin input (-), for .editorconfig and file name rules
  --limit <n>                       Report at most <n> violations
  --list-rules                      Print the ids of the available rules (porting is in progress)";

/// `Err` is a usage error message.
pub fn run(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|a| a == "--list-rules") {
        for provider in standard_rule_providers() {
            println!("{}", provider.rule_id().value());
        }
        return Ok(0);
    }
    let parsed = parse_lint_args(args)?;
    Ok(KtlintCli::from_env().run_lint(&parsed))
}

pub fn parse_lint_args(args: &[String]) -> Result<KtlintArgs, String> {
    let mut parsed = KtlintArgs { relative: true, ..KtlintArgs::default() };
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
            "--baseline" => parsed.baseline_path = value()?,
            "--editorconfig" => parsed.editor_config_path = Some(value()?),
            "--stdin-name" => parsed.stdin_path = Some(value()?),
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
    if !parsed.stdin && parsed.arguments.is_empty() {
        parsed.arguments.push(".".to_owned());
    }
    Ok(parsed)
}
