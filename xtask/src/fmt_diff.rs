//! `cargo fmt-diff [meta|google|kotlinlang] [dir]`: formats every .kt/.kts under `dir` (default
//! `corpus/`) with ktrs_fmt and compares byte-for-byte with the real ktfmt's output, pre-built by
//! `tools/ktfmt-oracle/ktfmt-oracle.sh <style> <dir> target/ktfmt-oracle/<style>`. Files ktfmt
//! rejected must be rejected by us too. Mismatches go to target/fmt-diff-<style>.txt.

use std::{
    collections::HashSet,
    fs,
    panic::{self, AssertUnwindSafe},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT, format};

use crate::corpus_diff::collect;

enum Outcome {
    Same,
    Differs(String),
    BothRejected,
    RejectionMismatch(String),
    Panic(String),
    NoOracle,
}

struct FileResult {
    rel: String,
    outcome: Outcome,
    bytes: usize,
    time: Duration,
}

pub(crate) fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let style = args.first().map_or("meta", String::as_str);
    let options = style_options(style)?;
    let dir = args.get(1).map_or_else(|| root.join("corpus"), PathBuf::from);
    let dir = fs::canonicalize(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let oracle = root.join("target/ktfmt-oracle").join(style);
    if !oracle.exists() {
        let name = dir.file_name().unwrap().to_string_lossy();
        return Err(format!(
            "no oracle output; build it first (JVM, slow):\n  tools/ktfmt-oracle/ktfmt-oracle.sh {style} {name} target/ktfmt-oracle/{style}"
        ));
    }
    let rejected = rejected_files(&oracle);

    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();

    let next = AtomicUsize::new(0);
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let wall = Instant::now();
    let mut results: Vec<FileResult> = thread::scope(|s| {
        let workers: Vec<_> = (0..thread::available_parallelism().map_or(4, |n| n.get()))
            .map(|_| {
                s.spawn(|| {
                    let mut out = Vec::new();
                    while let Some(file) = files.get(next.fetch_add(1, Ordering::Relaxed)) {
                        out.push(check(&dir, &oracle, &rejected, &options, file));
                    }
                    out
                })
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().unwrap()).collect()
    });
    let wall = wall.elapsed();
    panic::set_hook(previous_hook);
    results.sort_by(|a, b| a.rel.cmp(&b.rel));

    let mut report = String::new();
    let (mut same, mut differs, mut both_rejected, mut mismatched, mut panics, mut missing) = (0, 0, 0, 0, 0, 0);
    for r in &results {
        let detail = match &r.outcome {
            Outcome::Same => { same += 1; continue }
            Outcome::BothRejected => { both_rejected += 1; continue }
            Outcome::NoOracle => { missing += 1; continue }
            Outcome::Differs(first) => { differs += 1; first.clone() }
            Outcome::RejectionMismatch(what) => { mismatched += 1; format!("  {what}") }
            Outcome::Panic(message) => { panics += 1; format!("  panic: {message}") }
        };
        report.push_str(&format!("{}\n{detail}\n\n", r.rel));
    }
    let report_path = root.join(format!("target/fmt-diff-{style}.txt"));
    fs::write(&report_path, &report).map_err(|e| e.to_string())?;

    let cpu: f64 = results.iter().map(|r| r.time.as_secs_f64()).sum();
    let mb = results.iter().map(|r| r.bytes).sum::<usize>() as f64 / 1e6;
    println!(
        "{style}: identical {same}/{}  differ {differs}  panic {panics}  rejected-mismatch {mismatched}  both-rejected {both_rejected}/{}  no-oracle {missing}",
        files.len(),
        rejected.len()
    );
    println!(
        "formatted {mb:.1} MB in {cpu:.2} CPU-s = {:.1} MB/s per core ({:.1} MB/s wall)",
        mb / cpu,
        mb / wall.as_secs_f64()
    );
    if !report.is_empty() {
        println!("first differences: {}", report_path.display());
    }
    Ok(())
}

/// The CLI's `--meta-style` / `--google-style` / `--kotlinlang-style` presets.
fn style_options(style: &str) -> Result<FormattingOptions, String> {
    match style {
        "meta" => Ok(META_FORMAT),
        "google" => Ok(GOOGLE_FORMAT),
        "kotlinlang" => Ok(KOTLINLANG_FORMAT),
        _ => Err(format!("unknown style {style}; expected meta|google|kotlinlang")),
    }
}

/// Relative paths (with `/`) of the files ktfmt rejected: `.failed`, plus the `<path>:L:C: error:` lines
/// of `.stderr`, which ktfmt prints with the host's absolute path (`.failed` misses those on Windows).
fn rejected_files(oracle: &Path) -> HashSet<String> {
    let prefix = format!("{}/", oracle.to_string_lossy().replace('\\', "/").trim_start_matches("//?/").to_lowercase());
    let failed = fs::read_to_string(oracle.join(".failed")).unwrap_or_default();
    let stderr = fs::read_to_string(oracle.join(".stderr")).unwrap_or_default();
    let from_failed = failed.lines().map(|l| l.trim().trim_start_matches("./").to_owned());
    let from_stderr = stderr.lines().filter_map(|line| {
        let end = [".kt:", ".kts:"].iter().filter_map(|ext| line.find(ext).map(|i| i + ext.len() - 1)).min()?;
        let path = line[..end].replace('\\', "/");
        let lower = path.to_lowercase();
        let start = lower.find(&prefix).map_or(0, |i| i + prefix.len());
        Some(path[start..].trim_start_matches("./").to_owned())
    });
    from_failed.chain(from_stderr).filter(|p| !p.is_empty()).collect()
}

fn check(dir: &Path, oracle: &Path, rejected: &HashSet<String>, options: &FormattingOptions, file: &Path) -> FileResult {
    let rel = file.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/");
    let mut result = FileResult { rel, outcome: Outcome::NoOracle, bytes: 0, time: Duration::ZERO };
    let Ok(expected) = fs::read_to_string(oracle.join(&result.rel)) else { return result };
    let Ok(text) = fs::read_to_string(file) else { return result };
    result.bytes = text.len();
    let oracle_rejected = rejected.contains(&result.rel);

    let start = Instant::now();
    // The CLI strips a UTF-8 BOM before formatting.
    let code = text.strip_prefix('\u{feff}').unwrap_or(&text);
    // An already-formatted file is not rewritten, so its BOM survives.
    let formatted = panic::catch_unwind(AssertUnwindSafe(|| {
        format(code, options).map(|out| if out == code { text.clone() } else { out })
    }));
    result.time = start.elapsed();
    result.outcome = match (formatted, oracle_rejected) {
        (Err(payload), _) => Outcome::Panic(
            payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic>".to_owned()),
        ),
        (Ok(Err(_)), true) => Outcome::BothRejected,
        (Ok(Ok(_)), true) => Outcome::RejectionMismatch("ktfmt rejects this file, we formatted it".to_owned()),
        (Ok(Err(e)), false) => Outcome::RejectionMismatch(format!("we rejected: {e}")),
        (Ok(Ok(actual)), false) if actual == expected => Outcome::Same,
        (Ok(Ok(actual)), false) => Outcome::Differs(first_difference(&expected, &actual)),
    };
    result
}

fn first_difference(expected: &str, actual: &str) -> String {
    let (mut e, mut a) = (expected.split('\n'), actual.split('\n'));
    for line in 1.. {
        match (e.next(), a.next()) {
            (x, y) if x == y && x.is_some() => {}
            (x, y) => {
                let show = |s: Option<&str>| s.map_or("<eof>".to_owned(), |s| format!("{s:?}"));
                return format!("  line {line}\n  expected: {}\n  actual:   {}", show(x), show(y));
            }
        }
    }
    unreachable!()
}
