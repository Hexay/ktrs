//! `cargo run -p ktrs-detekt --release --example detekt_probe -- <in-dir> <out-dir> [--all-rules] [--threads N]`:
//! the Rust side of tools/detekt-oracle/detekt-probe.sh. Writes `<out-dir>/rows.tsv` in the oracle's layout
//! (files in path order, rows in analyzer order), `failed.tsv` (`<file>\tcrash\t<panic>`: the exception that
//! would abort the jar's run) and `run.txt`. Compare the two sides with `cargo detekt-diff`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{env, fs, panic, process, thread};

use ktrs_detekt::config::{load_configuration, workaround_configuration};
use ktrs_detekt::engine::create_analyzer;
use ktrs_detekt::probe::{escape, kotlin_files, rows_of_file};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--") && a.parse::<usize>().is_err()).collect();
    let [in_dir, out_dir] = positional.as_slice() else {
        eprintln!("usage: detekt_probe <in-dir> <out-dir> [--all-rules] [--threads N]");
        process::exit(2);
    };
    let all_rules = args.iter().any(|a| a == "--all-rules");
    let threads = args
        .iter()
        .position(|a| a == "--threads")
        .and_then(|i| args.get(i + 1)?.parse().ok())
        .unwrap_or_else(|| thread::available_parallelism().map_or(4, |n| n.get()));

    let base_path = fs::canonicalize(in_dir.as_str()).expect("the input directory exists");
    let out_dir = PathBuf::from(out_dir.as_str());
    fs::create_dir_all(&out_dir).expect("the output directory can be created");
    let config = workaround_configuration(load_configuration(&[]).expect("no config to load"), all_rules, false, false);
    let analyzer = create_analyzer(base_path.clone(), &config);
    let files = kotlin_files(&base_path);

    let started = Instant::now();
    let next = AtomicUsize::new(0);
    panic::set_hook(Box::new(|_| {}));
    let mut results: Vec<(usize, Result<Vec<String>, String>)> = thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.max(1))
            .map(|_| {
                thread::Builder::new()
                    .stack_size(256 << 20)
                    .spawn_scoped(scope, || {
                        let mut out = Vec::new();
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(file) = files.get(index) else { return out };
                            out.push((index, rows_of_file(&analyzer, file)));
                        }
                    })
                    .unwrap()
            })
            .collect();
        workers.into_iter().flat_map(|worker| worker.join().unwrap()).collect()
    });
    let seconds = started.elapsed().as_secs_f64();
    results.sort_by_key(|(index, _)| *index);

    let (mut rows, mut failed, mut issues) = (String::new(), String::new(), 0);
    for (index, result) in &results {
        let rel = files[*index].strip_prefix(&base_path).unwrap().to_string_lossy().replace('\\', "/");
        match result {
            Ok(file_rows) => {
                issues += file_rows.len();
                file_rows.iter().for_each(|row| rows.push_str(&format!("{rel}\t{row}\n")));
            }
            Err(message) => failed.push_str(&format!("{rel}\tcrash\t{}\n", escape(message))),
        }
    }
    fs::write(out_dir.join("rows.tsv"), rows).unwrap();
    if !failed.is_empty() {
        fs::write(out_dir.join("failed.tsv"), &failed).unwrap();
    }
    fs::write(out_dir.join("run.txt"), format!("files={}\nissues={issues}\nseconds={seconds}\nthreads={threads}\n", files.len())).unwrap();
    println!(
        "detekt probe: {} files, {issues} issues, {} failed, {seconds:.2} s on {threads} threads, {} rules",
        files.len(),
        failed.lines().count(),
        analyzer.rules().len()
    );
}
