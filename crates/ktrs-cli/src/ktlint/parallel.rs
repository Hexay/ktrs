//! `Sequence<Callable<T>>.parallel(cb)` of `KtlintCommandLine`: items are processed on one worker per
//! core and their results handed to `report` in item order. Like upstream's lazy `takeWhile`, workers
//! stop taking items once `stop()` holds (checked before each item, so a few in-flight items finish).
//! Workers are named like the JVM's first `Executors` pool threads, which ktlint's log lines show.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

pub const WORKER_THREAD_PREFIX: &str = "pool-1-thread-";

pub fn parallel<I: Sync, T: Send>(
    items: &[I],
    stop: impl Fn() -> bool + Sync,
    work: impl Fn(&I) -> T + Sync,
    report: impl FnMut(&I, T) + Send,
) {
    let next = AtomicUsize::new(0);
    let done: Mutex<(usize, Vec<Option<T>>, _)> = Mutex::new((0, items.iter().map(|_| None).collect(), report));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(items.len().max(1));
    std::thread::scope(|s| {
        for n in 1..=threads {
            let worker = || {
                loop {
                    if stop() {
                        break;
                    }
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    let Some(item) = items.get(i) else { break };
                    let result = work(item);
                    let mut done = done.lock().unwrap();
                    let (next_to_report, results, report) = &mut *done;
                    results[i] = Some(result);
                    while let Some(result) = results.get_mut(*next_to_report).and_then(Option::take) {
                        report(&items[*next_to_report], result);
                        *next_to_report += 1;
                    }
                }
            };
            std::thread::Builder::new()
                .name(format!("{WORKER_THREAD_PREFIX}{n}"))
                .spawn_scoped(s, worker)
                .expect("failed to spawn a worker thread");
        }
    });
}
