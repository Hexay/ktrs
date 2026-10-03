//! The native `ktrs` command. `ktrs fmt` maps its flags onto ktfmt's [`ParsedArgs`] and runs the
//! same [`Main`], so output, messages and exit codes stay those of the `ktfmt` drop-in; `ktrs lint`
//! does the same over the `ktlint` drop-in (`crate::ktrs_lint`); `ktrs ktlint` is that drop-in itself.

use std::io;
use std::path::Path;

use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT};

use crate::ktfmt::{KTFMT_VERSION, Main, ParsedArgs, editor_config_resolver};

const HELP_TEMPLATE: &str = "\
ktrs - fast Kotlin tooling

Usage:
  ktrs fmt [OPTIONS] [PATH ...]    Format .kt/.kts files in place (default PATH: .)
  ktrs fmt [OPTIONS] -             Format stdin to stdout
  ktrs lint [OPTIONS] [PATH ...]   Check .kt/.kts files with ktlint's rules (default PATH: .); exit 1
                                     on violations. Rules are being ported: `ktrs lint --list-rules`
  ktrs lint [OPTIONS] -            Check stdin (with --format: fixed code to stdout)
  ktrs serve                      Format requests framed on stdin until it closes (for build tools;
                                     protocol: crates/ktrs-cli/src/serve.rs)
  ktrs ktlint [ARGS ...]          Exactly the `ktlint` drop-in, flags and exit codes as ktlint's CLI
                                     (for build tools that bundle only `ktrs`)
  ktrs --version

Format options:
  --style <meta|google|kotlinlang>  Code style (default: meta)
  --check                           Don't write; list files that would change and exit 1 if any
  --keep-unused-imports             Leave unused imports in place
  --editorconfig                    Apply .editorconfig overrides (max_line_length, indent sizes,
                                      ktfmt_trailing_comma_management_strategy); for stdin, at
                                      --stdin-name
  --stdin-name <name>               Name (path) of the stdin input, for messages and .editorconfig
  -v, --verbose                     Report each formatted file

{LINT_HELP}

`ktfmt` and `ktlint` binaries with those tools' exact flags ship alongside, for existing scripts
and integrations.";

fn help() -> String {
    HELP_TEMPLATE.replace("{LINT_HELP}", crate::ktrs_lint::HELP)
}

pub fn run(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("fmt") => match parse_fmt_args(&args[1..]) {
            Ok(parsed) => {
                ktrs_syntax::caught_panic::silence_caught_panics();
                Main::new(io::stdin(), io::stdout(), io::stderr()).run_parsed(&parsed)
            }
            Err(message) => {
                eprintln!("error: {message}\n\n{}", help());
                2
            }
        },
        Some("lint") => crate::ktrs_lint::run(&args[1..]).unwrap_or_else(|message| {
            eprintln!("error: {message}\n\n{}", help());
            2
        }),
        // No `java_launcher` wildcard expansion: callers pass literal paths, not a shell's command line.
        Some("ktlint") => crate::ktlint::main(&args[1..]),
        Some("serve") if args.len() == 1 => crate::serve::run(io::stdin().lock(), io::stdout().lock()),
        Some("--version" | "-V") => {
            println!("ktrs {} (formats like ktfmt {KTFMT_VERSION})", env!("CARGO_PKG_VERSION"));
            0
        }
        Some("help" | "--help" | "-h") => {
            println!("{}", help());
            0
        }
        _ => {
            eprintln!("{}", help());
            2
        }
    }
}

pub fn parse_fmt_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut parsed = ParsedArgs {
        file_names: Vec::new(),
        formatting_options: META_FORMAT,
        dry_run: false,
        set_exit_if_changed: false,
        stdin_name: None,
        editor_config: false,
        quiet: true,
    };
    let mut keep_unused_imports = false;
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
    // Unlike ktfmt 0.64 (the `ktfmt` binary), resolve .editorconfig for stdin at --stdin-name, as
    // ktfmt's next release will: editors format buffers through stdin.
    if let (true, true, Some(name)) = (reads_stdin, parsed.editor_config, &parsed.stdin_name) {
        parsed.formatting_options =
            editor_config_resolver::resolve_formatting_options(Path::new(name), &parsed.formatting_options);
    }
    Ok(parsed)
}

pub(crate) fn style(name: &str) -> Result<FormattingOptions, String> {
    match name {
        "meta" => Ok(META_FORMAT),
        "google" => Ok(GOOGLE_FORMAT),
        "kotlinlang" => Ok(KOTLINLANG_FORMAT),
        _ => Err(format!("unknown style '{name}' (expected meta, google or kotlinlang)")),
    }
}
