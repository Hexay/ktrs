//! `cargo run -p ktrs_parser --release --example bench [dir] [reps] [filter]`: single-threaded
//! lex + parse throughput over a corpus, plus the files with the worst time per KB.
//!
//! Each file is timed best-of-`reps` in thread CPU cycles (Windows) or wall time (elsewhere), so
//! a busy machine skews the result much less than `cargo corpus-diff`'s summed wall times do.
//! Tree drop is excluded, as in `corpus-diff`.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use ktrs_parser::{FileKind, parse_file};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = PathBuf::from(args.first().map_or("corpus", String::as_str));
    let reps: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let filter = args.get(2).cloned().unwrap_or_default();
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.retain(|f| f.to_string_lossy().contains(&filter));
    files.sort();

    let clock = Clock::calibrate();
    let (mut lex, mut parse, mut bytes) = (0.0, 0.0, 0usize);
    let mut per_file = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default().replace("\r\n", "\n");
        let kind = FileKind::from_file_name(&file.to_string_lossy());
        lex += clock.best_of(reps, || drop(std::hint::black_box(ktrs_lexer::tokenize(&text))));
        let t = clock.best_of(reps, || std::hint::black_box(parse_file(&text, kind)));
        parse += t;
        bytes += text.len();
        per_file.push((t, text.len(), file));
    }
    let mb = bytes as f64 / 1e6;
    println!("{} files, {mb:.1} MB", files.len());
    println!("lex only:   {lex:.3} s = {:.1} MB/s", mb / lex);
    println!("parse_file: {parse:.3} s = {:.1} MB/s", mb / parse);
    per_file.retain(|f| f.1 >= 2000);
    per_file.sort_by(|a, b| (b.0 / b.1 as f64).total_cmp(&(a.0 / a.1 as f64)));
    println!("worst ms per 10 KB (files >= 2 KB):");
    for (t, len, file) in per_file.iter().take(8) {
        let name = file.strip_prefix(&dir).unwrap_or(file).display();
        println!("  {:6.2} ms/10KB {:7.2} ms {:6.1} KB  {name}", t * 1e7 / *len as f64, t * 1e3, *len as f64 / 1e3);
    }
}

/// Thread CPU time in seconds: cycles scaled by a calibrated rate, so time spent descheduled
/// doesn't count. Falls back to wall time off Windows.
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

    /// Seconds of the fastest of `reps` runs of `f`, excluding dropping its result.
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
