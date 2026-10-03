//! `ktrs serve`: a long-lived formatter for build tools (the Java shim in `java/`), so a build pays
//! for process startup once instead of once per file.
//!
//! Protocol v1, over stdin/stdout. Every message is a frame: a 4-byte big-endian length, then that
//! many bytes. The server sends one frame on start, `ktrs-serve 1 <version>`, then answers each
//! request frame with one response frame, in order, until stdin closes (exit 0).
//!
//! A request is UTF-8 `key=value` header lines, an empty line, then the source code:
//!
//! | key | values | default |
//! |---|---|---|
//! | `style` | `meta`, `google`, `kotlinlang` | `meta` |
//! | `max-width`, `block-indent`, `continuation-indent` | positive integers | the style's |
//! | `remove-unused-imports` | `true`, `false` | `true` |
//! | `trailing-commas` | `none`, `only_add`, `complete` | the style's |
//! | `path` | the file's path, for `.editorconfig` and to prefix messages | none |
//! | `editorconfig` | `true`, `false`: apply `.editorconfig` at `path` over the above | `false` |
//!
//! A response is `status=ok` or `status=error`, `changed=true|false` (ok only), an empty line, then
//! the formatted code or the error message (ktfmt's CLI wording, one line per diagnostic). Unknown
//! keys and bad values are errors for that request only; a broken frame ends the server (exit 2).

use std::io::{self, Read, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use ktrs_fmt::{FormatError, FormattingOptions, TrailingCommaManagementStrategy};

use crate::ktfmt::editor_config_resolver;
use crate::ktrs::style;

pub const PROTOCOL_VERSION: u32 = 1;
/// Larger frames are a broken client, not a Kotlin file.
const MAX_FRAME: usize = 256 << 20;
const UTF8_BOM: char = '\u{feff}';

pub fn run(input: impl Read, output: impl Write) -> i32 {
    let (mut input, mut output) = (io::BufReader::new(input), io::BufWriter::new(output));
    let hello = format!("ktrs-serve {PROTOCOL_VERSION} {}", env!("CARGO_PKG_VERSION"));
    let result = write_frame(&mut output, hello.as_bytes()).and_then(|()| {
        while let Some(request) = read_frame(&mut input)? {
            write_frame(&mut output, &respond(&request))?;
        }
        Ok(())
    });
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ktrs serve: {e}");
            2
        }
    }
}

fn read_frame(input: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut length = [0; 4];
    match input.read_exact(&mut length) {
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        result => result?,
    }
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("frame of {length} bytes")));
    }
    let mut frame = vec![0; length];
    input.read_exact(&mut frame)?;
    Ok(Some(frame))
}

fn write_frame(output: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    output.write_all(&(payload.len() as u32).to_be_bytes())?;
    output.write_all(payload)?;
    output.flush()
}

fn respond(request: &[u8]) -> Vec<u8> {
    match format_request(&String::from_utf8_lossy(request)) {
        Ok((formatted, changed)) => format!("status=ok\nchanged={changed}\n\n{formatted}").into_bytes(),
        Err(message) => format!("status=error\n\n{message}").into_bytes(),
    }
}

/// The formatted code and whether it differs from the input.
fn format_request(request: &str) -> Result<(String, bool), String> {
    let (header, code) = match request.strip_prefix('\n') {
        Some(code) => ("", code),
        None => request.split_once("\n\n").ok_or("request has no empty line after its header")?,
    };
    let request = Request::parse(header)?;
    let code = code.strip_prefix(UTF8_BOM).unwrap_or(code);
    let options = match &request.path {
        Some(path) if request.editorconfig => editor_config_resolver::resolve_formatting_options(Path::new(path), &request.options),
        _ => request.options,
    };
    let name = request.path.as_deref();
    match catch_unwind(AssertUnwindSafe(|| ktrs_fmt::format(code, &options))) {
        Ok(Ok(formatted)) => {
            let changed = formatted != code;
            Ok((formatted, changed))
        }
        Ok(Err(e)) => Err(error_message(name, &e)),
        Err(_) => Err(located(name, " internal error in ktrs (please report it with this file)")),
    }
}

struct Request {
    options: FormattingOptions,
    path: Option<String>,
    editorconfig: bool,
}

impl Request {
    fn parse(header: &str) -> Result<Request, String> {
        let fields: Vec<(&str, &str)> = header
            .lines()
            .map(|line| line.split_once('=').ok_or_else(|| format!("header line without '=': {line}")))
            .collect::<Result<_, _>>()?;
        // The style first: the other keys override its values.
        let style_name = fields.iter().rev().find(|(key, _)| *key == "style").map_or("meta", |(_, value)| value);
        let mut request = Request { options: style(style_name)?, path: None, editorconfig: false };
        for (key, value) in fields {
            let options = &mut request.options;
            match key {
                "style" => {}
                "max-width" => options.max_width = positive(key, value)?,
                "block-indent" => options.block_indent = positive(key, value)?,
                "continuation-indent" => options.continuation_indent = positive(key, value)?,
                "remove-unused-imports" => options.remove_unused_imports = boolean(key, value)?,
                "trailing-commas" => options.trailing_comma_management_strategy = trailing_commas(value)?,
                "path" => request.path = Some(value.to_owned()),
                "editorconfig" => request.editorconfig = boolean(key, value)?,
                _ => return Err(format!("unknown request key '{key}'")),
            }
        }
        Ok(request)
    }
}

fn positive(key: &str, value: &str) -> Result<i32, String> {
    value.parse().ok().filter(|&n: &i32| n > 0).ok_or_else(|| format!("{key} must be a positive integer, got '{value}'"))
}

fn boolean(key: &str, value: &str) -> Result<bool, String> {
    value.parse().map_err(|_| format!("{key} must be true or false, got '{value}'"))
}

fn trailing_commas(value: &str) -> Result<TrailingCommaManagementStrategy, String> {
    match value {
        "none" => Ok(TrailingCommaManagementStrategy::None),
        "only_add" => Ok(TrailingCommaManagementStrategy::OnlyAdd),
        "complete" => Ok(TrailingCommaManagementStrategy::Complete),
        _ => Err(format!("trailing-commas must be none, only_add or complete, got '{value}'")),
    }
}

/// ktfmt's CLI wording (`Main.format`'s catch blocks), without the stack trace. Without a `path`
/// the message is ktfmt's exception text alone, as callers like Spotless add the file name.
fn error_message(name: Option<&str>, error: &FormatError) -> String {
    match error {
        FormatError::Parse(e) => located(name, &e.to_string()),
        FormatError::Formatting(e) => e.diagnostics().iter().map(|d| located(name, &d.to_string())).collect::<Vec<_>>().join("\n"),
        FormatError::Formatter(_) | FormatError::Runtime(_) | FormatError::MissedTokens(_) => located(name, &format!(" {error}")),
    }
}

fn located(name: Option<&str>, message: &str) -> String {
    match name {
        Some(name) => format!("{name}:{message}"),
        None => message.trim_start().to_string(),
    }
}
