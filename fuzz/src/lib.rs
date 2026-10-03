//! Shared pieces of the fuzz targets and `kgen`: input decoding, the parse-error filter, the Java-exception panic
//! filter and the token-level splice mutator. Method and findings: research/24-fuzzing.md.

mod splice;

pub use splice::{boundaries as token_boundaries, crossover, crossover_text, mutate, mutate_in_place};

use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

use ktrs_parser::FileKind;

/// Past this `(` nesting depth the parser's type-argument backtracking (`a<((((`) is exponential in time and, in
/// ktrs, memory: a known finding (research/24-fuzzing.md) that would otherwise drown every target in timeouts/OOMs.
pub const KNOWN_SLOW_PAREN_DEPTH: usize = 10;

/// The text a target runs on: UTF-8 (the CLIs decode lossily, which only adds U+FFFD) and within
/// [`KNOWN_SLOW_PAREN_DEPTH`]; anything else is skipped.
pub fn fuzz_input(data: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(data).ok()?;
    let mut depth = 0usize;
    for b in text.bytes() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth > KNOWN_SLOW_PAREN_DEPTH {
            return None;
        }
    }
    Some(text)
}

/// `StringUtilRt.convertLineSeparators`: what every caller does before handing text to the parser.
pub fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Whether the text parses without a single error element, as `kind`.
pub fn parses_cleanly(text: &str, kind: FileKind) -> bool {
    !ktrs_parser::parse_file(&normalize_newlines(text), kind).has_errors()
}

/// A panic message that mimics a Java exception (`IllegalArgumentException: ...`, `AssertionError`): ktrs raises
/// these on purpose where ktfmt/ktlint throw, so one is not a finding by itself (tools/fuzz/diff.sh checks the jar
/// throws too).
pub fn is_java_exception(message: &str) -> bool {
    let name = message.split([':', ' ', '\n']).next().unwrap_or("");
    let simple = name.rsplit('.').next().unwrap_or(name);
    !simple.is_empty()
        && simple.chars().all(|c| c.is_ascii_alphanumeric())
        && (simple.ends_with("Exception") || simple.ends_with("Error") || simple == "Throwable")
}

thread_local! {
    static TOLERATING: Cell<bool> = const { Cell::new(false) };
}

/// Runs `f`, returning `None` if it panicked with a Java-exception message, or at a location listed in
/// `KTRS_FUZZ_KNOWN` (comma-separated `file.rs:line` suffixes: findings already reported, so the fuzzer can get past
/// them). Any other panic, caught downstream (ktrs-lint's rule boundary) or not, reaches libFuzzer's abort hook.
pub fn tolerate_java_panics<R>(f: impl FnOnce() -> R) -> Option<R> {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let known: Vec<String> = std::env::var("KTRS_FUZZ_KNOWN")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let payload = info.payload();
            let message = payload.downcast_ref::<&str>().copied().or_else(|| payload.downcast_ref::<String>().map(String::as_str));
            let at = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
            let expected = message.is_some_and(is_java_exception) || known.iter().any(|k| at.ends_with(k.as_str()));
            if !(TOLERATING.get() && expected) {
                previous(info);
            }
        }));
    });
    let outer = TOLERATING.replace(true);
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    TOLERATING.set(outer);
    result.ok()
}

/// Installs the splice mutator and crossover in a fuzz target (needs `libfuzzer_sys` in the target's deps).
#[macro_export]
macro_rules! splice_mutators {
    () => {
        ::libfuzzer_sys::fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
            $crate::mutate_in_place(data, size, max_size, seed, ::libfuzzer_sys::fuzzer_mutate)
        });
        ::libfuzzer_sys::fuzz_crossover!(|a: &[u8], b: &[u8], out: &mut [u8], seed: u32| {
            $crate::crossover(a, b, out, seed)
        });
    };
}
