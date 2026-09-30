//! Port of ktfmt's `cli/ParsedArgs.kt` (v0.64).

use std::fs;

use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT};

pub const KTFMT_VERSION: &str = "0.64";

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedArgs {
    pub file_names: Vec<String>,
    pub formatting_options: FormattingOptions,
    /// Run the formatter without writing changes to any files; print the path of any files that
    /// would be changed.
    pub dry_run: bool,
    /// Return exit code 1 if any formatting changes are detected.
    pub set_exit_if_changed: bool,
    /// File name to report when formating code from stdin.
    pub stdin_name: Option<String>,
    pub editor_config: bool,
    /// Suppress all non-error output.
    pub quiet: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseResult {
    Ok(ParsedArgs),
    ShowMessage(String),
    Error(String),
}

pub const HELP_TEXT: &str = "\
ktfmt - command line Kotlin source code pretty-printer

Usage:
  ktfmt [OPTIONS] <File1.kt> <File2.kt> ...
  ktfmt @ARGFILE

ktfmt formats Kotlin source code files in-place, reporting for each file whether the
formatting succeeded or failed on standard error. If none of the style options are
passed, Meta's style is used.

Alternatively, ktfmt can read Kotlin source code from standard input and write the\x20
formatted result on standard output.

Example:
     $ ktfmt --kotlinlang-style Main.kt src/Parser.kt
     Done formatting Main.kt
     Error formatting src/Parser.kt: @@@ERROR@@@; skipping.
\x20  \x20
Commands options:
  -h, --help                        Show this help message
  -v, --version                     Show version
  -n, --dry-run                     Don't write to files, only report files which
                                        would have changed
  --meta-style                      Use 2-space block indenting (default)
  --google-style                    Google internal style (2 spaces)
  --kotlinlang-style                Kotlin language guidelines style (4 spaces)
  --stdin-name=<name>               Name to report when formatting code from stdin
  --set-exit-if-changed             Sets exit code to 1 if any input file was not
                                        formatted/touched
  --do-not-remove-unused-imports    Leaves all imports in place, even if not used
  --enable-editorconfig             Enable .editorconfig overrides for supported formatting options (limited)
                                        see https://github.com/facebook/ktfmt/blob/main/README.md
  --quiet                           Suppress all non-error output

ARGFILE:
  If the only argument begins with '@', the remainder of the argument is treated
  as the name of a file to read options and arguments from, one per line.
\x20\x20
  e.g.
      $ cat arg-file.txt
      --google-style
      -n
      File1.kt
      File2.kt
      $ ktfmt @arg-file1.txt
      Done formatting File1.kt
      Done formatting File2.kt
";

/// `processArgs`: expands a lone `@ARGFILE`. Upstream lets the read's `IOException` escape `main`.
pub fn process_args(args: &[String]) -> Result<ParseResult, std::io::Error> {
    if args.len() == 1 && args[0].starts_with('@') {
        let text = fs::read(&args[0][1..])?;
        let lines: Vec<String> = String::from_utf8_lossy(&text).lines().map(str::to_owned).collect();
        return Ok(parse_options(&lines));
    }
    Ok(parse_options(args))
}

/// `parseOptions` parses command-line arguments passed to ktfmt.
pub fn parse_options(args: &[String]) -> ParseResult {
    let mut file_names: Vec<String> = Vec::new();
    let mut formatting_options = META_FORMAT;
    let mut dry_run = false;
    let mut set_exit_if_changed = false;
    let mut remove_unused_imports = true;
    let mut stdin_name: Option<String> = None;
    let mut editor_config = false;
    let mut quiet = false;

    let has = |flag: &str| args.iter().any(|a| a == flag);
    if has("--help") || has("-h") {
        return ParseResult::ShowMessage(HELP_TEXT.to_owned());
    }
    if has("--version") || has("-v") {
        return ParseResult::ShowMessage(format!("ktfmt version {KTFMT_VERSION}"));
    }

    for arg in args {
        match arg.as_str() {
            "--meta-style" => formatting_options = META_FORMAT,
            "--google-style" => formatting_options = GOOGLE_FORMAT,
            "--kotlinlang-style" => formatting_options = KOTLINLANG_FORMAT,
            "--dry-run" | "-n" => dry_run = true,
            "--set-exit-if-changed" => set_exit_if_changed = true,
            "--do-not-remove-unused-imports" => remove_unused_imports = false,
            "--enable-editorconfig" => editor_config = true,
            "--quiet" => quiet = true,
            _ if arg.starts_with("--stdin-name=") => match parse_key_value_arg("--stdin-name", arg) {
                Some(value) => stdin_name = Some(value.to_owned()),
                None => return ParseResult::Error(format!("Found option '{arg}', expected '--stdin-name=<value>'")),
            },
            _ if arg.starts_with("--") || arg.starts_with('@') => {
                return ParseResult::Error(format!("Unexpected option: {arg}"));
            }
            _ => file_names.push(arg.clone()),
        }
    }

    if file_names.iter().any(|f| f == "-") {
        // We're reading from stdin
        if file_names.len() > 1 {
            // Kotlin's `list - element` drops only the first occurrence.
            let mut files_except_stdin: Vec<&str> = file_names.iter().map(String::as_str).collect();
            let first_stdin = files_except_stdin.iter().position(|f| *f == "-").expect("contains '-'");
            files_except_stdin.remove(first_stdin);
            return ParseResult::Error(format!(
                "Cannot read from stdin and files in same run. Found stdin specifier '-' and files {} ",
                files_except_stdin.join(", ")
            ));
        }
    } else if stdin_name.is_some() {
        return ParseResult::Error("--stdin-name can only be specified when reading from stdin".to_owned());
    }

    ParseResult::Ok(ParsedArgs {
        file_names,
        formatting_options: FormattingOptions { remove_unused_imports, ..formatting_options },
        dry_run,
        set_exit_if_changed,
        stdin_name,
        editor_config,
        quiet,
    })
}

/// Upstream's `takeIf { parts[0] == key || parts.size == 2 }` holds whenever there is a `=`.
fn parse_key_value_arg<'a>(_key: &str, arg: &'a str) -> Option<&'a str> {
    arg.split_once('=').map(|(_, value)| value)
}
