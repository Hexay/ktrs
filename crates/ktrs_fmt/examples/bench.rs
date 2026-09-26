//! `cargo run -p ktrs_fmt --release --example bench [dir] [reps]`: single-thread META_FORMAT throughput
//! over every .kt/.kts under `dir` (default `corpus/`), plus the slowest files.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::{env, fs};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "kt" || e == "kts") {
            out.push(path);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = PathBuf::from(args.get(1).map_or("corpus", String::as_str));
    let reps: u32 = args.get(2).map_or(1, |r| r.parse().unwrap());
    let mut files = Vec::new();
    collect(&dir, &mut files);
    let texts: Vec<(PathBuf, String)> =
        files.into_iter().filter_map(|f| fs::read_to_string(&f).ok().map(|t| (f, t))).collect();
    let mut times: Vec<(Duration, &Path)> = Vec::new();
    let mut total = Duration::ZERO;
    let mut bytes = 0usize;
    for (path, text) in &texts {
        let start = Instant::now();
        for _ in 0..reps {
            let _ = ktrs_fmt::format(text, &ktrs_fmt::META_FORMAT);
        }
        let elapsed = start.elapsed() / reps;
        total += elapsed;
        bytes += text.len();
        times.push((elapsed, path));
    }
    println!("{} files, {:.1} MB in {:.2}s = {:.2} MB/s", texts.len(), bytes as f64 / 1e6, total.as_secs_f64(), bytes as f64 / 1e6 / total.as_secs_f64());
    times.sort_by(|a, b| b.0.cmp(&a.0));
    for (time, path) in times.iter().take(10) {
        println!("{:>8.1} ms  {}", time.as_secs_f64() * 1e3, path.display());
    }
}
