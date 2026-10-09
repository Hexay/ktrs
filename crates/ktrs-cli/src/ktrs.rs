//! The native `ktrs` command: `ktrs fmt` (`crate::ktrs_fmt`) and `ktrs lint` (`crate::ktrs_lint`) run the engines
//! of the `ktfmt` and `ktlint` drop-ins under their own flags; `ktrs ktlint` is that drop-in itself.

use std::io;

use crate::ktfmt::KTFMT_VERSION;

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
  ktrs lsp                        Language server on stdin/stdout: ktlint diagnostics, fixes and
                                     suppressions, ktfmt or ktlint formatting (settings:
                                     crates/ktrs-lsp/src/lib.rs)
  ktrs ktlint [ARGS ...]          Exactly the `ktlint` drop-in, flags and exit codes as ktlint's CLI
                                     (for build tools that bundle only `ktrs`)
  ktrs migrate [--write] [PATH]   Switch the build's ktfmt/ktlint plugins to the ktrs drop-ins: a diff,
                                     exit 1 if any (`ktrs migrate --help`)
  ktrs --version

{FMT_HELP}

{LINT_HELP}

`ktfmt` and `ktlint` binaries with those tools' exact flags ship alongside, for existing scripts
and integrations.";

fn help() -> String {
    HELP_TEMPLATE.replace("{FMT_HELP}", crate::ktrs_fmt::HELP).replace("{LINT_HELP}", crate::ktrs_lint::HELP)
}

pub fn run(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("fmt") => crate::ktrs_fmt::run(&args[1..]).unwrap_or_else(usage_error),
        Some("lint") => crate::ktrs_lint::run(&args[1..]).unwrap_or_else(usage_error),
        // No `java_launcher` wildcard expansion: callers pass literal paths, not a shell's command line.
        Some("ktlint") => crate::ktlint::main(&args[1..]),
        Some("migrate") => crate::migrate::run(&args[1..], &mut io::stdout().lock(), &mut io::stderr().lock()),
        Some("serve") if args.len() == 1 => crate::serve::run(io::stdin().lock(), io::stdout().lock()),
        // `--stdio` is what VS Code's client and others pass; stdio is the only transport.
        Some("lsp") if args[1..].iter().all(|a| a == "--stdio") => ktrs_lsp::run_stdio(),
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

fn usage_error(message: String) -> i32 {
    eprintln!("error: {message}\n\n{}", help());
    2
}
