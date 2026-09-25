//! `cargo corpus-diff [dir]`: parses every .kt/.kts under `dir` (default `corpus/`) and diffs our
//! PSI dump against the real compiler's, pre-built by `psi-dump.sh tree <dir> target/oracle/<dir name>`.
//! Mismatches go to target/corpus-diff.txt. Use a release build for meaningful throughput numbers.

use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::psi_dump;

enum Outcome {
    Same,
    Differs(String),
    NoOracle,
}

pub(crate) fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let dir = args.first().map_or_else(|| root.join("corpus"), PathBuf::from);
    let dir = fs::canonicalize(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let name = dir.file_name().unwrap().to_string_lossy();
    let oracle = root.join("target/oracle").join(&*name);
    if !oracle.exists() {
        return Err(format!(
            "no oracle dumps; build them first (JVM, slow):\n  tools/psi-dump/psi-dump.sh tree {name} target/oracle/{name}"
        ));
    }

    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();

    let chunk = files.len().div_ceil(thread::available_parallelism().map_or(4, |n| n.get())).max(1);
    let mut results: Vec<FileResult> = thread::scope(|s| {
        let handles: Vec<_> = files
            .chunks(chunk)
            .map(|part| {
                let (dir, oracle) = (&dir, &oracle);
                s.spawn(move || part.iter().map(|f| check(dir, oracle, f)).collect::<Vec<_>>())
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });

    let mut report = String::new();
    let (mut same, mut differs, mut missing) = (0, 0, 0);
    for FileResult { file, outcome, .. } in &results {
        match outcome {
            Outcome::Same => same += 1,
            Outcome::NoOracle => missing += 1,
            Outcome::Differs(first) => {
                differs += 1;
                report.push_str(&format!("{}\n{first}\n\n", file.strip_prefix(&dir).unwrap().display()));
            }
        }
    }
    let report_path = root.join("target/corpus-diff.txt");
    fs::write(&report_path, &report).map_err(|e| e.to_string())?;

    let secs: f64 = results.iter().map(|r| r.parse_time.as_secs_f64()).sum();
    let mb = results.iter().map(|r| r.bytes).sum::<usize>() as f64 / 1e6;
    println!("identical {same}/{}  differ {differs}  no-oracle {missing}", files.len());
    println!("parsed {mb:.1} MB in {secs:.2} CPU-s = {:.1} MB/s per core", mb / secs);
    results.sort_by_key(|r| std::cmp::Reverse(r.parse_time));
    println!("slowest:");
    for r in results.iter().take(8) {
        let kb = r.bytes as f64 / 1e3;
        let ms = r.parse_time.as_secs_f64() * 1e3;
        println!("  {ms:8.1} ms {kb:7.1} KB  {}", r.file.strip_prefix(&dir).unwrap().display());
    }
    if differs > 0 {
        println!("first differences: {}", report_path.display());
    }
    Ok(())
}

struct FileResult {
    file: PathBuf,
    outcome: Outcome,
    bytes: usize,
    parse_time: Duration,
}

fn check(dir: &Path, oracle: &Path, file: &Path) -> FileResult {
    let rel = file.strip_prefix(dir).unwrap();
    let mut result = FileResult { file: file.to_path_buf(), outcome: Outcome::NoOracle, bytes: 0, parse_time: Duration::ZERO };
    let Ok(expected) = fs::read_to_string(oracle.join(format!("{}.txt", rel.display()))) else {
        return result;
    };
    // psi-dump normalizes CRLF only; unlike the fixtures, trailing newlines are kept.
    let text = fs::read_to_string(file).unwrap_or_default().replace("\r\n", "\n");
    let name = file.file_name().unwrap().to_string_lossy();
    let start = Instant::now();
    let parse = parse_file(&text, FileKind::from_file_name(&name));
    result.parse_time = start.elapsed();
    result.bytes = text.len();
    let actual = psi_dump(&parse, &name);
    result.outcome = first_difference(expected.replace("\r\n", "\n").trim_end(), actual.trim_end());
    result
}

fn first_difference(expected: &str, actual: &str) -> Outcome {
    let (mut e, mut a) = (expected.lines(), actual.lines());
    for line in 1.. {
        match (e.next(), a.next()) {
            (None, None) => return Outcome::Same,
            (x, y) if x == y => {}
            (x, y) => {
                return Outcome::Differs(format!(
                    "  line {line}\n  expected: {}\n  actual:   {}",
                    x.unwrap_or("<eof>"),
                    y.unwrap_or("<eof>")
                ));
            }
        }
    }
    unreachable!()
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "kt" || e == "kts") {
            out.push(path);
        }
    }
}
