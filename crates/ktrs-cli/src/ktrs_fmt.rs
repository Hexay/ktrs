//! `ktrs fmt`: maps its flags onto ktfmt's [`ParsedArgs`] and runs the same [`Main`], so output, messages and exit
//! codes stay those of the `ktfmt` drop-in. Its own: `--changed-since` (`crate::changed_since`; a run it leaves
//! without files exits 0) and `--check --reporter github` (`crate::github_annotations`).

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT};

use crate::changed_since::{ChangedFiles, OPTION as CHANGED_SINCE};
use crate::github_annotations::{Annotations, REPORTER_ID, UnformattedFiles};
use crate::ktfmt::{Main, ParsedArgs, editor_config_resolver, expand_args_to_file_names};

pub const HELP: &str = "\
Format options:
  --style <meta|google|kotlinlang>  Code style (default: meta)
  --check                           Don't write; list files that would change and exit 1 if any
  --reporter <plain|github>         With --check: github prints the files as GitHub Actions
                                      annotations (`::error file=...`)
  --changed-since <ref>             Only files that differ from the merge base of <ref> and HEAD,
                                      uncommitted and untracked ones included (needs git)
  --keep-unused-imports             Leave unused imports in place
  --editorconfig                    Apply .editorconfig overrides (max_line_length, indent sizes,
                                      ktfmt_trailing_comma_management_strategy); for stdin, at
                                      --stdin-name
  --stdin-name <name>               Name (path) of the stdin input, for messages and .editorconfig
  -v, --verbose                     Report each formatted file";

#[derive(Clone, Debug)]
pub struct FmtArgs {
    pub parsed: ParsedArgs,
    pub changed_since: Option<String>,
    pub github: bool,
}

/// `Err` is a usage error message.
pub fn run(args: &[String]) -> Result<i32, String> {
    let fmt = parse_fmt_args(args)?;
    let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    ktrs_syntax::caught_panic::silence_caught_panics();
    run_with(&fmt, &working_dir, io::stdin(), io::stdout(), io::stderr())
}

/// [`run`] with `working_dir` as the place of the git repository and the base of reported paths (file
/// arguments still resolve against the process's directory).
pub fn run_with<R, O, E>(fmt: &FmtArgs, working_dir: &Path, input: R, out: O, err: E) -> Result<i32, String>
where
    R: Read + Send,
    O: Write + Send,
    E: Write + Send,
{
    let mut parsed = fmt.parsed.clone();
    if let Some(reference) = &fmt.changed_since {
        let changed = ChangedFiles::since(reference, working_dir)?;
        let files = expand_args_to_file_names(&parsed.file_names);
        // No Kotlin files at all stays ktfmt's error.
        if !files.is_empty() {
            parsed.file_names = files.iter().filter(|file| changed.contains(file)).map(|file| file.display().to_string()).collect();
            if parsed.file_names.is_empty() {
                return Ok(0);
            }
        }
    }
    Ok(if fmt.github {
        Main::new(input, UnformattedFiles::new(out, Annotations::from_env(working_dir)), err).run_parsed(&parsed)
    } else {
        Main::new(input, out, err).run_parsed(&parsed)
    })
}

pub fn parse_fmt_args(args: &[String]) -> Result<FmtArgs, String> {
    let mut parsed = ParsedArgs {
        file_names: Vec::new(),
        formatting_options: META_FORMAT,
        dry_run: false,
        set_exit_if_changed: false,
        stdin_name: None,
        editor_config: false,
        quiet: true,
    };
    let (mut keep_unused_imports, mut changed_since, mut github) = (false, None, false);
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let (flag, inline_value) = match arg.split_once('=') {
            Some((flag, value)) if arg.starts_with("--") => (flag, Some(value.to_owned())),
            _ => (arg.as_str(), None),
        };
        let mut value = || inline_value.clone().or_else(|| args.next().cloned()).ok_or(format!("{flag} needs a value"));
        match flag {
            "--style" => parsed.formatting_options = style(&value()?)?,
            "--check" => (parsed.dry_run, parsed.set_exit_if_changed) = (true, true),
            "--reporter" => github = reporter_is_github(&value()?)?,
            CHANGED_SINCE => changed_since = Some(value()?),
            "--keep-unused-imports" => keep_unused_imports = true,
            "--editorconfig" => parsed.editor_config = true,
            "--stdin-name" => parsed.stdin_name = Some(value()?),
            "-v" | "--verbose" => parsed.quiet = false,
            "-" => parsed.file_names.push(arg.clone()),
            _ if flag.starts_with('-') => return Err(format!("unknown option {arg}")),
            _ => parsed.file_names.push(arg.clone()),
        }
    }
    parsed.formatting_options.remove_unused_imports = !keep_unused_imports;
    if parsed.file_names.is_empty() {
        parsed.file_names.push(".".to_owned());
    }
    let reads_stdin = parsed.file_names.iter().any(|f| f == "-");
    if reads_stdin && parsed.file_names.len() > 1 {
        return Err("cannot read from stdin and files in the same run".to_owned());
    }
    if parsed.stdin_name.is_some() && !reads_stdin {
        return Err("--stdin-name can only be used when reading from stdin (-)".to_owned());
    }
    if reads_stdin && changed_since.is_some() {
        return Err(format!("{CHANGED_SINCE} can not be used when reading from stdin (-)"));
    }
    if github && !parsed.dry_run {
        return Err(format!("--reporter {REPORTER_ID} needs --check"));
    }
    // Unlike ktfmt 0.64 (the `ktfmt` binary), resolve .editorconfig for stdin at --stdin-name, as
    // ktfmt's next release will: editors format buffers through stdin.
    if let (true, true, Some(name)) = (reads_stdin, parsed.editor_config, &parsed.stdin_name) {
        parsed.formatting_options =
            editor_config_resolver::resolve_formatting_options(Path::new(name), &parsed.formatting_options);
    }
    Ok(FmtArgs { parsed, changed_since, github })
}

fn reporter_is_github(name: &str) -> Result<bool, String> {
    match name {
        "plain" => Ok(false),
        REPORTER_ID => Ok(true),
        _ => Err(format!("unknown reporter '{name}' (expected plain or {REPORTER_ID})")),
    }
}

pub(crate) fn style(name: &str) -> Result<FormattingOptions, String> {
    match name {
        "meta" => Ok(META_FORMAT),
        "google" => Ok(GOOGLE_FORMAT),
        "kotlinlang" => Ok(KOTLINLANG_FORMAT),
        _ => Err(format!("unknown style '{name}' (expected meta, google or kotlinlang)")),
    }
}
