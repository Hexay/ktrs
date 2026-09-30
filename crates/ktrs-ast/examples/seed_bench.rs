//! `cargo run -p ktrs-ast --release --example seed_bench [dir] [reps]`: arena seeding (`Ast::from_parse`)
//! against parsing, per file best-of-`reps` in thread CPU cycles, like ktrs-parser's `bench` example (whose
//! clock this copies). Go/no-go (b) of research/11 "Prototype first": seeding <= 25% of parse.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::{env, fs};

use ktrs_ast::Ast;
use ktrs_parser::{FileKind, parse_file};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let dir = PathBuf::from(args.first().map_or("corpus", String::as_str));
    let reps: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();

    let clock = Clock::calibrate();
    let (mut parse, mut seed, mut bytes, mut nodes) = (0.0, 0.0, 0usize, 0usize);
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default().replace("\r\n", "\n");
        let kind = FileKind::from_file_name(&file.to_string_lossy());
        parse += clock.best_of(reps, || std::hint::black_box(parse_file(&text, kind)));
        let parsed = parse_file(&text, kind);
        seed += clock.best_of(reps, || std::hint::black_box(Ast::from_parse(&parsed)));
        bytes += text.len();
        nodes += parsed.tree.len();
    }
    let mb = bytes as f64 / 1e6;
    println!("{} files, {mb:.1} MB, {nodes} nodes", files.len());
    println!("parse_file:      {parse:.3} s = {:.1} MB/s", mb / parse);
    println!("Ast::from_parse: {seed:.3} s = {:.1} MB/s", mb / seed);
    println!("seed / parse = {:.1}%", 100.0 * seed / parse);
}

/// Thread CPU time in seconds (see ktrs-parser/examples/bench.rs).
struct Clock {
    seconds_per_cycle: Option<f64>,
}

impl Clock {
    fn calibrate() -> Clock {
        let seconds_per_cycle = thread_cycles().map(|_| {
            (0..3)
                .map(|_| {
                    let (c0, t0) = (thread_cycles().unwrap(), Instant::now());
                    while t0.elapsed() < Duration::from_millis(30) {}
                    t0.elapsed().as_secs_f64() / (thread_cycles().unwrap() - c0) as f64
                })
                .fold(f64::INFINITY, f64::min)
        });
        Clock { seconds_per_cycle }
    }

    fn best_of<R>(&self, reps: u32, mut f: impl FnMut() -> R) -> f64 {
        (0..reps.max(1))
            .map(|_| {
                let (c0, t0) = (thread_cycles(), Instant::now());
                let result = f();
                let t = match (self.seconds_per_cycle, c0, thread_cycles()) {
                    (Some(scale), Some(c0), Some(c1)) => (c1 - c0) as f64 * scale,
                    _ => t0.elapsed().as_secs_f64(),
                };
                drop(result);
                t
            })
            .fold(f64::INFINITY, f64::min)
    }
}

#[cfg(windows)]
fn thread_cycles() -> Option<u64> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentThread() -> isize;
        fn QueryThreadCycleTime(thread: isize, cycles: *mut u64) -> i32;
    }
    let mut cycles = 0;
    // SAFETY: plain Win32 calls on the current thread's pseudo-handle.
    (unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut cycles) } != 0).then_some(cycles)
}

#[cfg(not(windows))]
fn thread_cycles() -> Option<u64> {
    None
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
