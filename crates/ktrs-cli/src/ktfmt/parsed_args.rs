//! Port of ktfmt's `cli/ParsedArgs.kt` (v0.65); `--experimental-engine`: see the module docs.

use std::fs;

use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT, Range, RangeSet};

pub(crate) use super::java_int::to_int_or_null;

pub const KTFMT_VERSION: &str = "0.65";

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedArgs {
    pub file_names: Vec<String>,
    pub formatting_options: FormattingOptions,
    /// Run the formatter without writing changes to any files; print the path of any files that
    /// would be changed.
    pub dry_run: bool,
    /// Return exit code 1 if any formatting changes are detected.
    pub set_exit_if_changed: bool,
    /// Path to report for stdin and, when EditorConfig is enabled, use for configuration lookup.
    pub stdin_name: Option<String>,
    pub editor_config: bool,
    /// Suppress all non-error output.
    pub quiet: bool,
    /// Zero-indexed line ranges to format, using closed-open bounds, e.g. [0, 3) and [6, 7).
    pub line_ranges: RangeSet,
    /// Zero-indexed character ranges (UTF-16 units) to format, using closed-open bounds.
    pub character_ranges: RangeSet,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseResult {
    Ok(ParsedArgs),
    ShowMessage(String),
    Error(String),
}

/// What `processArgs` throws instead of returning: the JVM prints it and exits with 1.
#[derive(Debug)]
pub enum ArgsException {
    /// The argfile read's `IOException`.
    Io(std::io::Error),
    /// Guava's `Range.closedOpen` on a negative `--length`: the exception's message.
    IllegalArgument(String),
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
  --stdin-name=<path>               Path to report for stdin and, when EditorConfig is
                                        enabled, use for configuration lookup
  --lines=<lines>                   Line range(s) to format, like 5 or 1:12,14.
                                        May be used multiple times.
  --offset=<offset>                 Character offset to format, paired with --length.
                                        May be used multiple times.
  --length=<length>                 Character length to format, paired with --offset.
                                        May be used multiple times. 0 formats the whole
                                        line containing the given --offset.
  --set-exit-if-changed             Sets exit code to 1 if any input file was not
                                        formatted/touched
  --do-not-remove-unused-imports    Leaves all imports in place, even if not used
  --enable-editorconfig             Enable .editorconfig overrides for supported formatting options (limited)
                                        see https://github.com/Kotlin/ktfmt/blob/main/README.md
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

/// `processArgs`: expands a lone `@ARGFILE`.
pub fn process_args(args: &[String]) -> Result<ParseResult, ArgsException> {
    if args.len() == 1 && args[0].starts_with('@') {
        let text = fs::read(&args[0][1..]).map_err(ArgsException::Io)?;
        let lines: Vec<String> = String::from_utf8_lossy(&text).lines().map(str::to_owned).collect();
        return parse_options(&lines);
    }
    parse_options(args)
}

/// `parseOptions` parses command-line arguments passed to ktfmt.
pub fn parse_options(args: &[String]) -> Result<ParseResult, ArgsException> {
    let mut file_names: Vec<String> = Vec::new();
    let mut formatting_options = META_FORMAT;
    let mut dry_run = false;
    let mut set_exit_if_changed = false;
    let mut remove_unused_imports = true;
    let mut stdin_name: Option<String> = None;
    let mut editor_config = false;
    let mut quiet = false;
    let mut line_ranges = RangeSet::create();
    let mut offsets: Vec<i32> = Vec::new();
    let mut lengths: Vec<i32> = Vec::new();

    let has = |flag: &str| args.iter().any(|a| a == flag);
    if has("--help") || has("-h") {
        return Ok(ParseResult::ShowMessage(HELP_TEXT.to_owned()));
    }
    if has("--version") || has("-v") {
        return Ok(ParseResult::ShowMessage(format!("ktfmt version {KTFMT_VERSION}")));
    }

    let error = |message: String| Ok(ParseResult::Error(message));
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--meta-style" => formatting_options = META_FORMAT,
            "--google-style" => formatting_options = GOOGLE_FORMAT,
            "--kotlinlang-style" => formatting_options = KOTLINLANG_FORMAT,
            "--dry-run" | "-n" => dry_run = true,
            "--set-exit-if-changed" => set_exit_if_changed = true,
            "--do-not-remove-unused-imports" => remove_unused_imports = false,
            "--enable-editorconfig" => editor_config = true,
            "--quiet" => quiet = true,
            "--experimental-engine" => {
                eprintln!("You are using undocumented and unstable feature. We do not recommend doing so");
                return error("--experimental-engine is not supported by ktrs".to_owned());
            }
            _ if arg.starts_with("--stdin-name=") => match parse_key_value_arg("--stdin-name", arg) {
                Some(value) => stdin_name = Some(value.to_owned()),
                None => return error(format!("Found option '{arg}', expected '--stdin-name=<value>'")),
            },
            _ if arg.starts_with("--line") => {
                let key = arg.split_once('=').map_or(arg, |(key, _)| key);
                if key != "--lines" && key != "--line" {
                    return error(format!("Unexpected option: {key}"));
                }
                let Some(value) = key_value(args, &mut i) else {
                    return error(format!("required value was not provided for: {key}"));
                };
                if let Err(message) = parse_line_ranges(&mut line_ranges, value) {
                    return error(message);
                }
            }
            _ if arg.starts_with("--offset") || arg.starts_with("--length") => {
                let name = &arg[..8];
                let key = arg.split_once('=').map_or(arg, |(key, _)| key);
                if key != name {
                    return error(format!("Unexpected option: {key}"));
                }
                let Some(value) = key_value(args, &mut i) else {
                    return error(format!("required value was not provided for: {key}"));
                };
                let Some(number) = to_int_or_null(value) else {
                    return error(format!("invalid integer value for {key}: {value}"));
                };
                if name == "--offset" { offsets.push(number) } else { lengths.push(number) }
            }
            _ if arg.starts_with("--") || arg.starts_with('@') => {
                return error(format!("Unexpected option: {arg}"));
            }
            _ => file_names.push(arg.to_owned()),
        }
        i += 1;
    }

    if file_names.iter().any(|f| f == "-") {
        // We're reading from stdin
        if file_names.len() > 1 {
            // Kotlin's `list - element` drops only the first occurrence.
            let mut files_except_stdin: Vec<&str> = file_names.iter().map(String::as_str).collect();
            let first_stdin = files_except_stdin.iter().position(|f| *f == "-").expect("contains '-'");
            files_except_stdin.remove(first_stdin);
            return error(format!(
                "Cannot read from stdin and files in same run. Found stdin specifier '-' and files {} ",
                files_except_stdin.join(", ")
            ));
        }
    } else if stdin_name.is_some() {
        return error("--stdin-name can only be specified when reading from stdin".to_owned());
    }

    if offsets.len() != lengths.len() {
        return error("--offset and --length flags must be provided in matching pairs".to_owned());
    }

    let character_ranges = character_ranges(&offsets, &lengths).map_err(ArgsException::IllegalArgument)?;

    if (!line_ranges.is_empty() || !character_ranges.is_empty()) && file_names.len() != 1 {
        return error("partial formatting is only supported for a single file".to_owned());
    }

    Ok(ParseResult::Ok(ParsedArgs {
        file_names,
        formatting_options: FormattingOptions { remove_unused_imports, ..formatting_options },
        dry_run,
        set_exit_if_changed,
        stdin_name,
        editor_config,
        quiet,
        line_ranges,
        character_ranges,
    }))
}

/// The value of `key=value` at `args[*i]`, or of `key value`, consuming the next argument (upstream's `nextValue`).
fn key_value<'a>(args: &'a [String], i: &mut usize) -> Option<&'a str> {
    match args[*i].split_once('=') {
        Some((_, value)) => Some(value),
        None => {
            *i += 1;
            args.get(*i).map(String::as_str)
        }
    }
}

/// Upstream's `takeIf { parts[0] == key || parts.size == 2 }` holds whenever there is a `=`.
fn parse_key_value_arg<'a>(_key: &str, arg: &'a str) -> Option<&'a str> {
    arg.split_once('=').map(|(_, value)| value)
}

/// The ranges of matching `--offset`/`--length` pairs; `Err` is Guava's message for a negative length.
pub(crate) fn character_ranges(offsets: &[i32], lengths: &[i32]) -> Result<RangeSet, String> {
    let mut character_ranges = RangeSet::create();
    for (&offset, &length) in offsets.iter().zip(lengths) {
        let length = if length == 0 { 1 } else { length };
        character_ranges.add(closed_open(offset, offset.wrapping_add(length))?);
    }
    Ok(character_ranges)
}

pub(crate) fn parse_line_ranges(line_ranges: &mut RangeSet, line_ranges_arg: &str) -> Result<(), String> {
    for line_range in line_ranges_arg.split(',') {
        // Any `IllegalArgumentException`: a `NumberFormatException` or an inverted range.
        let range = parse_line_range(line_range).ok_or(format!("invalid line range for --lines: {line_ranges_arg}"))?;
        line_ranges.add(range);
    }
    Ok(())
}

fn parse_line_range(arg: &str) -> Option<Range> {
    let parts: Vec<&str> = arg.split(':').collect();
    match parts.as_slice() {
        [line] => {
            let line = to_int_or_null(line)?.wrapping_sub(1);
            closed_open(line, line.wrapping_add(1)).ok()
        }
        [line0, line1] => {
            let line0 = to_int_or_null(line0)?.wrapping_sub(1);
            let line1 = to_int_or_null(line1)?.wrapping_sub(1);
            closed_open(line0, line1.wrapping_add(1)).ok()
        }
        _ => None,
    }
}

/// `Range.closedOpen`, with Guava's message for an inverted range.
fn closed_open(lower: i32, upper: i32) -> Result<Range, String> {
    if lower > upper {
        return Err(format!("Invalid range: [{lower}..{upper})"));
    }
    Ok(Range::closed_open(lower, upper))
}
