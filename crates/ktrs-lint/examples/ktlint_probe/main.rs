//! Rust side of the ktlint mutated-tree oracle (tools/ktlint-oracle, `KtlintProbe.kt`): same output layout,
//! so the two out-dirs diff file by file.
//!
//!   cargo ktlint-probe <src-dir> <out-dir> [--rules a,b] [--dumps] [--no-lint] [--threads N]
//!   cargo ktlint-probe compare <jvm-out-dir> <rust-out-dir> [src-dir]
//!
//! Out-dir: fmt/<rel> (formatted text, only when changed), mut/<rel>.p<N>.txt (psiToString of the mutated
//! tree after pass N, when it changed; --dumps), diff/<rel>.p<N>.diff (mutated vs fresh parse, when they
//! diverge), passes.tsv, format.tsv (format callback rows, emit order, with pass), lint.tsv, failed.tsv,
//! summary.txt. Rules default to every ported rule; unported ids are an error.

mod compare;
mod probe;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs, process, thread};

use ktrs_lint::KtLintRuleEngine;
use ktrs_lint::editorconfig::KtlintVersion;
use ktrs_lint::rule_provider::rule_providers_in;
use ktrs_lint::rules::{standard_rule_provider, standard_rule_providers};
use probe::FileResult;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

struct Options {
    src: PathBuf,
    out: PathBuf,
    rules: Vec<String>,
    dumps: bool,
    lint: bool,
    threads: usize,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "compare") && (3..=4).contains(&args.len()) {
        let (jvm, rust) = (Path::new(&args[1]), Path::new(&args[2]));
        let src = args.get(3).map_or_else(|| jvm.join("src"), PathBuf::from);
        process::exit(compare::compare(jvm, rust, &src));
    }
    let Some(opts) = parse_args(&args) else {
        eprintln!("usage: ktlint_probe <src-dir> <out-dir> [--rules a,b] [--dumps] [--no-lint] [--threads N] | compare <jvm-out> <rust-out> [src]");
        process::exit(2);
    };
    let engine = KtLintRuleEngine::new(
        opts.rules.iter().map(|id| standard_rule_provider(id).unwrap_or_else(|| panic!("rule {id} is not ported"))).collect(),
    );
    let mut files = Vec::new();
    collect(&opts.src, &opts.src, &mut files);
    files.sort();
    let t0 = std::time::Instant::now();
    let results = run_all(&engine, &opts, &files);
    write_tables(&opts, &results, t0.elapsed().as_secs_f64());
}

fn parse_args(args: &[String]) -> Option<Options> {
    let mut opts = Options {
        src: PathBuf::from(args.first()?),
        out: PathBuf::from(args.get(1)?),
        rules: rule_providers_in(&standard_rule_providers(), KtlintVersion::V2_0).iter().map(|p| p.rule_id().value().to_owned()).collect(),
        dumps: false,
        lint: true,
        threads: thread::available_parallelism().map_or(1, |n| n.get()),
    };
    let mut rest = args[2..].iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--rules" => opts.rules = rest.next()?.split(',').map(str::to_owned).collect(),
            "--dumps" => opts.dumps = true,
            "--no-lint" => opts.lint = false,
            "--threads" => opts.threads = rest.next()?.parse().ok()?,
            _ => return None,
        }
    }
    Some(opts)
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else if path.extension().is_some_and(|e| e == "kt" || e == "kts") {
            out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

/// Deep trees recurse through the traversal, like the JVM oracle (which gives its threads 512 MB stacks).
fn run_all(engine: &KtLintRuleEngine, opts: &Options, files: &[String]) -> Vec<FileResult> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(files.len()));
    thread::scope(|s| {
        for _ in 0..opts.threads.max(1) {
            thread::Builder::new()
                .stack_size(256 << 20)
                .spawn_scoped(s, || {
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(rel) = files.get(i) else { break };
                        let result = probe::process(engine, &opts.src.join(rel), rel, opts.dumps, opts.lint);
                        results.lock().unwrap().push((i, result));
                    }
                })
                .unwrap();
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, r)| r).collect()
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn write_tables(opts: &Options, results: &[FileResult], wall: f64) {
    let out = &opts.out;
    let _ = fs::remove_dir_all(out);
    fs::create_dir_all(out).unwrap();
    let (mut passes, mut format, mut lint, mut failed) = (
        String::from("file\tpass\tchanged\tdiverged\n"),
        String::from("file\tpass\tline\tcol\trule\tauto\tdetail\n"),
        String::from("file\tline\tcol\trule\tauto\tdetail\n"),
        String::new(),
    );
    for r in results {
        if let Some(formatted) = &r.formatted {
            write(&out.join("fmt").join(&r.rel), formatted);
        }
        for p in &r.passes {
            passes.push_str(&format!("{}\t{}\t{}\t{}\n", r.rel, p.pass, u8::from(p.changed), u8::from(p.diverged)));
            if let Some(mutated) = p.mutated.as_ref().filter(|_| p.changed && opts.dumps) {
                // `psiToString` ends every line with '\n'; `psi_to_string` trims the last one, like `psi_dump`.
                write(&out.join(format!("mut/{}.p{}.txt", r.rel, p.pass)), &format!("{mutated}\n"));
            }
            if let (true, Some(mutated), Some(reparsed)) = (p.diverged, &p.mutated, &p.reparsed) {
                write(&out.join(format!("diff/{}.p{}.diff", r.rel, p.pass)), &compare::hunk(mutated, reparsed, 40));
            }
        }
        r.format.iter().for_each(|row| format.push_str(&format!("{}\t{row}\n", r.rel)));
        r.lint.iter().for_each(|row| lint.push_str(&format!("{}\t{row}\n", r.rel)));
        if let Some(failure) = &r.failure {
            failed.push_str(&format!("{}\t{failure}\n", r.rel));
        }
    }
    for (name, table) in [("passes.tsv", &passes), ("format.tsv", &format), ("lint.tsv", &lint), ("failed.tsv", &failed)] {
        write(&out.join(name), table);
    }
    let ok: Vec<&FileResult> = results.iter().filter(|r| r.failure.is_none()).collect();
    let summary = format!(
        "ktrs_lint probe (ktlint 2.0.0-ALPHA-4 port), rules={}\nfiles {}, failed {}\nfiles changed {}, diverged (any pass) {}\n\
         wall {wall:.1} s; per-file time sums: format {:.1} s (probe {:.1} s of it)\n",
        opts.rules.join(","),
        results.len(),
        results.len() - ok.len(),
        ok.iter().filter(|r| r.passes.iter().any(|p| p.changed)).count(),
        ok.iter().filter(|r| r.passes.iter().any(|p| p.diverged)).count(),
        ok.iter().map(|r| r.format_seconds).sum::<f64>(),
        ok.iter().map(|r| r.probe_seconds).sum::<f64>(),
    );
    write(&out.join("summary.txt"), &summary);
    print!("{summary}");
}
