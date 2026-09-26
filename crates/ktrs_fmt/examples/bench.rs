//! `cargo run -p ktrs_fmt --release --example bench [dir] [reps] [filter] [threads]`: META_FORMAT
//! throughput over every .kt/.kts under `dir` (default `corpus/`), plus the worst files per KB.
//!
//! Single-thread (default): each file is timed best-of-`reps` in thread CPU cycles (Windows), so a
//! busy machine skews the result much less than wall time. With `threads` > 1, formats the whole set
//! once on that many threads and reports wall time and summed per-thread CPU time instead.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = PathBuf::from(args.first().map_or("corpus", String::as_str));
    let reps: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let filter = args.get(2).cloned().unwrap_or_default();
    let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.retain(|f| f.to_string_lossy().contains(&filter));
    files.sort();
    let texts: Vec<(PathBuf, String)> = files
        .into_iter()
        .filter_map(|f| fs::read_to_string(&f).ok().map(|t| (f, t.strip_prefix('\u{feff}').unwrap_or(&t).to_owned())))
        .collect();
    let mb = texts.iter().map(|t| t.1.len()).sum::<usize>() as f64 / 1e6;
    let clock = Clock::calibrate();
    if threads > 1 {
        return run_parallel(&texts, threads, &clock, mb);
    }

    let (mut total, mut parse) = (0.0, 0.0);
    let mut per_file = Vec::new();
    for (path, text) in &texts {
        let t = clock.best_of(reps, || ktrs_fmt::format(text, &ktrs_fmt::META_FORMAT));
        parse += clock.best_of(reps, || ktrs_parser::parse_file(text, ktrs_parser::FileKind::Script));
        total += t;
        per_file.push((t, text.len(), path));
    }
    println!("{} files, {mb:.1} MB in {total:.2} CPU-s = {:.2} MB/s", texts.len(), mb / total);
    // The parser is a fixed yardstick measured under the same load: a steadier number to compare runs by.
    println!("one parse_file: {parse:.2} CPU-s = {:.1} MB/s; format = {:.2} parses", mb / parse, total / parse);
    per_file.retain(|f| f.1 >= 2000);
    per_file.sort_by(|a, b| (b.0 / b.1 as f64).total_cmp(&(a.0 / a.1 as f64)));
    println!("average {:.2} ms/10KB; worst (files >= 2 KB):", total * 1e7 / (mb * 1e6));
    for (t, len, file) in per_file.iter().take(8) {
        let name = file.strip_prefix(&dir).unwrap_or(file).display();
        println!("  {:6.2} ms/10KB {:8.2} ms {:6.1} KB  {name}", t * 1e7 / *len as f64, t * 1e3, *len as f64 / 1e3);
    }
}

fn run_parallel(texts: &[(PathBuf, String)], threads: usize, clock: &Clock, mb: f64) {
    let next = AtomicUsize::new(0);
    let wall = Instant::now();
    let cpu: f64 = thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    clock.best_of(1, || {
                        while let Some((_, text)) = texts.get(next.fetch_add(1, Ordering::Relaxed)) {
                            drop(ktrs_fmt::format(text, &ktrs_fmt::META_FORMAT));
                        }
                    })
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).sum()
    });
    let wall = wall.elapsed().as_secs_f64();
    println!("{threads} threads: {mb:.1} MB, wall {wall:.2}s = {:.2} MB/s; CPU {cpu:.2}s = {:.2} MB/s per core", mb / wall, mb / cpu);
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
