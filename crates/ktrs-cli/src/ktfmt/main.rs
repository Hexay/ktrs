//! Port of ktfmt's `cli/Main.kt` (v0.64): formats files in place (in parallel) or stdin to stdout.
//! Messages, exit codes and write decisions match ktfmt's (`tools/ktfmt-oracle/cli-diff.sh`).
//! Deviation: each file's messages are printed in file order, not in completion order.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use ktrs_fmt::FormatError;
use ktrs_syntax::caught_panic::catch_quietly;

use super::editor_config_resolver;
use super::parsed_args::{ParseResult, ParsedArgs, process_args};

const EXIT_CODE_FAILURE: i32 = 1;
const EXIT_CODE_SUCCESS: i32 = 0;
const UTF8_BOM: &str = "\u{feff}";
/// Java's `println` ends lines with the platform separator.
const LINE_SEPARATOR: &str = if cfg!(windows) { "\r\n" } else { "\n" };

const USAGE: &str = "\
Usage:
  ktfmt [OPTIONS] File1.kt File2.kt ...
  ktfmt @ARGFILE

For more details see `ktfmt --help`
";

pub struct Main<R, O, E> {
    input: Mutex<R>,
    out: Mutex<O>,
    err: Mutex<E>,
}

/// One file's output, buffered so files report in order.
#[derive(Default)]
struct Report {
    out: String,
    err: String,
}

impl Report {
    fn out(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push_str(LINE_SEPARATOR);
    }

    fn err(&mut self, line: &str) {
        self.err.push_str(line);
        self.err.push_str(LINE_SEPARATOR);
    }
}

impl<R: Read + Send, O: Write + Send, E: Write + Send> Main<R, O, E> {
    pub fn new(input: R, out: O, err: E) -> Self {
        Main { input: Mutex::new(input), out: Mutex::new(out), err: Mutex::new(err) }
    }

    /// The captured streams, for tests.
    pub fn into_streams(self) -> (O, E) {
        (self.out.into_inner().unwrap(), self.err.into_inner().unwrap())
    }

    pub fn run(&self, input_args: &[String]) -> i32 {
        let mut report = Report::default();
        let code = match process_args(input_args) {
            Ok(ParseResult::Ok(parsed_args)) => return self.run_parsed(&parsed_args),
            Ok(ParseResult::ShowMessage(message)) => {
                report.out(&message);
                EXIT_CODE_SUCCESS
            }
            Ok(ParseResult::Error(error_message)) => {
                report.err(&error_message);
                EXIT_CODE_FAILURE
            }
            // Upstream's argfile read throws out of `main`: the JVM reports it and exits with 1.
            Err(e) => {
                let file = Path::new(&input_args[0][1..]);
                report.err(&format!("Exception in thread \"main\" java.io.FileNotFoundException: {}", java_io_message(Some(file), &e)));
                EXIT_CODE_FAILURE
            }
        };
        self.flush(&report);
        code
    }

    pub fn run_parsed(&self, parsed_args: &ParsedArgs) -> i32 {
        if parsed_args.file_names.is_empty() {
            self.flush(&Report { err: format!("{USAGE}{LINE_SEPARATOR}"), ..Report::default() });
            return EXIT_CODE_FAILURE;
        }

        if parsed_args.file_names.len() == 1 && parsed_args.file_names[0] == "-" {
            // Format code read from stdin
            let mut report = Report::default();
            let result = self.format(None, parsed_args, &mut report);
            self.flush(&report);
            return match result {
                Ok(already_formatted) if !already_formatted && parsed_args.set_exit_if_changed => EXIT_CODE_FAILURE,
                Ok(_) => EXIT_CODE_SUCCESS,
                Err(()) => EXIT_CODE_FAILURE,
            };
        }

        let files = expand_args_to_file_names(&parsed_args.file_names);
        if files.is_empty() {
            self.flush(&Report { err: format!("Error: no .kt files found{LINE_SEPARATOR}"), ..Report::default() });
            return EXIT_CODE_FAILURE;
        }

        // `files.parallelStream().forEach`, one worker per core.
        let failed = AtomicBool::new(false);
        let next = AtomicUsize::new(0);
        let done: Mutex<(usize, Vec<Option<Report>>)> = Mutex::new((0, files.iter().map(|_| None).collect()));
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(files.len());
        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| {
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file) = files.get(i) else { break };
                        let mut report = Report::default();
                        match self.format(Some(file), parsed_args, &mut report) {
                            Ok(false) if parsed_args.set_exit_if_changed => failed.store(true, Ordering::Relaxed),
                            Ok(_) => {}
                            Err(()) => failed.store(true, Ordering::Relaxed),
                        }
                        let mut done = done.lock().unwrap();
                        let (next_to_print, reports) = &mut *done;
                        reports[i] = Some(report);
                        while let Some(report) = reports.get_mut(*next_to_print).and_then(Option::take) {
                            self.flush(&report);
                            *next_to_print += 1;
                        }
                    }
                });
            }
        });
        if failed.into_inner() { EXIT_CODE_FAILURE } else { EXIT_CODE_SUCCESS }
    }

    /// Formats `file`, or stdin when `None`; `Ok(true)` iff the input is valid and already formatted.
    /// `Err` is upstream's rethrown exception: IO, parse and formatting errors are reported first.
    fn format(&self, file: Option<&Path>, args: &ParsedArgs, report: &mut Report) -> Result<bool, ()> {
        let file_name = match file {
            Some(file) => java_file_name(file),
            None => args.stdin_name.clone().unwrap_or_else(|| "<stdin>".to_owned()),
        };
        let mut io_error = |e: io::Error| {
            report.err(&format!("Error formatting {file_name}: {}; skipping.", java_io_message(file, &e)));
        };

        let formatting_options = match file {
            Some(file) if args.editor_config => {
                editor_config_resolver::resolve_formatting_options(file, &args.formatting_options)
            }
            _ => args.formatting_options,
        };
        let bytes = match file {
            Some(file) => fs::read(file).map_err(&mut io_error)?,
            None => {
                let mut bytes = Vec::new();
                self.input.lock().unwrap().read_to_end(&mut bytes).map_err(&mut io_error)?;
                bytes
            }
        };
        let text = String::from_utf8_lossy(&bytes);
        let code = text.strip_prefix(UTF8_BOM).unwrap_or(&text);
        let formatted_code = match catch_quietly(|| ktrs_fmt::format(code, &formatting_options)) {
            Ok(Ok(formatted_code)) => formatted_code,
            Ok(Err(e)) => {
                report_error(report, &file_name, &e);
                return Err(());
            }
            // A formatter bug: like an unexpected JVM exception, it fails only this file.
            Err(_) => return Err(()),
        };
        let already_formatted = code == formatted_code;

        if args.dry_run {
            if !already_formatted {
                report.out(&file_name);
            }
        } else if let Some(file) = file {
            if !already_formatted {
                fs::write(file, &formatted_code).map_err(&mut io_error)?;
            }
            if !args.quiet {
                report.err(&format!("Done formatting {file_name}"));
            }
        } else {
            report.out.push_str(&formatted_code);
        }
        Ok(already_formatted)
    }

    fn flush(&self, report: &Report) {
        for (stream, text) in [(&self.out as &dyn Flush, &report.out), (&self.err as &dyn Flush, &report.err)] {
            if !text.is_empty() {
                stream.write_flush(text.as_bytes());
            }
        }
    }
}

trait Flush {
    fn write_flush(&self, bytes: &[u8]);
}

impl<W: Write> Flush for Mutex<W> {
    fn write_flush(&self, bytes: &[u8]) {
        let mut stream = self.lock().unwrap();
        let _ = stream.write_all(bytes).and_then(|()| stream.flush());
    }
}

/// The `catch` blocks for `ParseError` and `FormattingError`; other exceptions pass silently.
fn report_error(report: &mut Report, file_name: &str, error: &FormatError) {
    match error {
        FormatError::Parse(e) => report.err(&format!("{file_name}:{e}")),
        FormatError::Formatting(e) => {
            for diagnostic in e.diagnostics() {
                report.err(&format!("{file_name}:{diagnostic}"));
            }
            // `e.printStackTrace(err)`: its first line.
            report.err(&format!("com.google.googlejavaformat.FormattingError: {error}"));
        }
        FormatError::Formatter(_) | FormatError::Runtime(_) => {}
    }
}

/// `expandArgsToFileNames` expands `args` to a list of .kt files to format: a lone file as is,
/// otherwise every .kt/.kts file under each argument, in directory order (sorted here).
pub fn expand_args_to_file_names(args: &[String]) -> Vec<PathBuf> {
    if args.len() == 1 && Path::new(&args[0]).is_file() {
        return vec![PathBuf::from(&args[0])];
    }
    let mut result = Vec::new();
    for arg in args {
        walk_top_down(Path::new(arg), &mut result);
    }
    result
}

fn walk_top_down(path: &Path, result: &mut Vec<PathBuf>) {
    let Ok(metadata) = fs::metadata(path) else { return };
    if metadata.is_file() {
        // Kotlin's `File.extension`: whatever follows the name's last dot.
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        if matches!(name.rsplit_once('.'), Some((_, "kt" | "kts"))) {
            result.push(path.to_path_buf());
        }
    } else if metadata.is_dir() {
        let Ok(entries) = fs::read_dir(path) else { return };
        let mut children: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
        children.sort();
        for child in children {
            walk_top_down(&child, result);
        }
    }
}

/// `File.toString()`: Java normalizes separators to the platform's.
fn java_file_name(file: &Path) -> String {
    let name = file.to_string_lossy();
    if cfg!(windows) { name.replace('/', "\\") } else { name.into_owned() }
}

/// A Java `IOException` message: `path (reason)` for a file, the OS reason alone otherwise.
fn java_io_message(file: Option<&Path>, e: &io::Error) -> String {
    let message = e.to_string();
    let reason = message.split(" (os error").next().unwrap_or(&message).trim_end_matches('.');
    match file {
        Some(file) => format!("{} ({reason})", java_file_name(file)),
        None => reason.to_owned(),
    }
}
