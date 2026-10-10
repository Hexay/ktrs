//! `cargo run -p kt-syntax --release --example entry_cost [dir] [reps]`: what the facade's entry point costs on top of
//! the parser (normalization, the nesting guard, error indexing), over every .kt/.kts under `dir` (default
//! `corpus/`). Each file is timed best-of-`reps`, single-threaded.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::{env, fs};

use ktrs_parser::{FileKind, parse_file};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let dir = PathBuf::from(args.first().map_or("corpus", String::as_str));
    let reps: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();
    let (mut raw, mut facade, mut bytes, mut refused) = (Duration::ZERO, Duration::ZERO, 0, 0);
    for path in &files {
        let Ok(text) = fs::read_to_string(path) else { continue };
        let script = path.extension().is_some_and(|e| e == "kts");
        let normalized = text.strip_prefix('\u{feff}').unwrap_or(&text).replace("\r\n", "\n");
        bytes += normalized.len();
        raw += best_of(reps, || drop(parse_file(&normalized, if script { FileKind::Script } else { FileKind::Source })));
        let parse = if script { kt_syntax::parse_script } else { kt_syntax::parse };
        refused += usize::from(parse(&text).is_err());
        facade += best_of(reps, || drop(parse(&text)));
    }
    let mb = bytes as f64 / 1e6;
    println!("{} files, {mb:.1} MB, {refused} refused", files.len());
    println!("ktrs_parser::parse_file: {:.3} s = {:.1} MB/s", raw.as_secs_f64(), mb / raw.as_secs_f64());
    println!("kt_syntax::parse:        {:.3} s = {:.1} MB/s", facade.as_secs_f64(), mb / facade.as_secs_f64());
    println!("facade = {:.3} parses", facade.as_secs_f64() / raw.as_secs_f64());
}

fn best_of(reps: u32, mut f: impl FnMut()) -> Duration {
    (0..reps.max(1))
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed()
        })
        .min()
        .unwrap()
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("kt" | "kts")) {
            out.push(path);
        }
    }
}
