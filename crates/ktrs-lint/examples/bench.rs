//! `cargo run -p ktrs-lint --release --example bench [dir] [reps] [all|lint|format|rules] [filter]`: single-threaded
//! ktlint engine throughput over a corpus, every standard rule, ktlint_official defaults (no `.editorconfig`).
//!
//! `all`: per file, best-of-`reps` thread CPU time of `transformToAst` (parse + seed), `lint` and `format`
//! (autocorrect everything; `lint`/`format` time only that one), plus the files with the worst lint/format time per KB.
//! `rules`: lint and format with each rule alone, minus a baseline engine running `final-newline` alone.

use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use ktrs_lint::rule_provider::rule_providers_in;
use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::editorconfig::{CODE_STYLE_PROPERTY, EXPERIMENTAL_RULES_EXECUTION_PROPERTY, KtlintVersion, PropertyRef};
use ktrs_lint::{AutocorrectDecision, Code, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine, RuleV2Provider};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const BASELINE_RULE: &str = "standard:final-newline";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || run(&args))
        .unwrap()
        .join()
        .unwrap();
}

fn run(args: &[String]) {
    let dir = PathBuf::from(args.first().map_or("corpus", String::as_str));
    let reps: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let mode = args.get(2).map_or("all", String::as_str);
    let filter = args.get(3).cloned().unwrap_or_default();
    ktrs_lint::engine::set_verify_visited_types(std::env::var_os("KTRS_BENCH_VERIFY").is_some_and(|v| v == "1"));
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.retain(|f| f.to_string_lossy().contains(&filter));
    files.sort();
    let codes: Vec<(PathBuf, Code)> = files
        .into_iter()
        .map(|f| {
            let text = String::from_utf8_lossy(&fs::read(&f).unwrap_or_default()).into_owned();
            let script = f.extension().is_some_and(|e| e == "kts");
            let code = Code::from_snippet(&text, script);
            (f, code)
        })
        .filter(|(_, code)| KtLintRuleEngine::new(vec![baseline()]).transform_to_ast(code).is_ok())
        .collect();
    let bytes: usize = codes.iter().map(|(_, c)| c.content.len()).sum();
    println!("{} files (parse errors skipped), {:.1} MB", codes.len(), bytes as f64 / 1e6);
    let clock = Clock::calibrate();
    match mode {
        "rules" => per_rule(&clock, reps, &codes),
        _ => all(&clock, reps, mode, &codes, &dir, bytes),
    }
}

/// `KTRS_BENCH_STYLE=<code style>[-experimental]` overrides the code style (as `cargo lint-diff`'s run names);
/// `KTRS_BENCH_VERIFY=1` checks every rule's `visited_types` (slow).
fn engine(providers: Vec<RuleV2Provider>) -> KtLintRuleEngine {
    let Ok(style) = std::env::var("KTRS_BENCH_STYLE") else {
        return KtLintRuleEngine::new(providers);
    };
    let (style, experimental) = style.strip_suffix("-experimental").map_or((style.as_str(), false), |s| (s, true));
    let mut properties = vec![(PropertyRef::from(&*CODE_STYLE_PROPERTY), Some(style.to_owned()))];
    if experimental {
        properties.push((PropertyRef::from(&*EXPERIMENTAL_RULES_EXECUTION_PROPERTY), Some("enabled".to_owned())));
    }
    KtLintRuleEngine::with_editor_config(providers, EditorConfigDefaults::empty(), EditorConfigOverride::from(properties))
}

fn baseline() -> RuleV2Provider {
    standard_rule_providers().into_iter().find(|p| p.rule_id().value() == BASELINE_RULE).unwrap()
}

fn all(clock: &Clock, reps: u32, mode: &str, codes: &[(PathBuf, Code)], dir: &Path, bytes: usize) {
    let engine = engine(standard_rule_providers());
    let (mut parse, mut lint, mut format) = (0.0, 0.0, 0.0);
    let (mut lint_errors, mut changed) = (0usize, 0usize);
    // Output identity across builds: every lint error, format callback and formatted text.
    let mut digest = DefaultHasher::new();
    let mut per_file = Vec::new();
    let (do_lint, do_format) = (mode != "format", mode != "lint");
    for (file, code) in codes {
        parse += clock.best_of(reps, || engine.transform_to_ast(code));
        let l = if do_lint { clock.best_of(reps, || lint_once(&engine, code)) } else { 0.0 };
        let f = if do_format { clock.best_of(reps, || format_once(&engine, code)) } else { 0.0 };
        if do_lint {
            let _ = engine.lint(code, &mut |e| {
                lint_errors += 1;
                e.hash(&mut digest);
            });
        }
        if do_format {
            let formatted = engine.format(code, &mut |e| {
                e.hash(&mut digest);
                AutocorrectDecision::AllowAutocorrect
            });
            changed += usize::from(formatted.as_ref().is_ok_and(|t| *t != code.content));
            format!("{formatted:?}").hash(&mut digest);
        }
        (lint, format) = (lint + l, format + f);
        per_file.push((l, f, code.content.len(), file));
    }
    let mb = bytes as f64 / 1e6;
    println!("output digest: {:016x}", digest.finish());
    println!("parse+seed: {parse:.3} s = {:.2} MB/s", mb / parse);
    println!("lint:       {lint:.3} s = {:.2} MB/s  ({lint_errors} errors) = {:.2} parses", mb / lint, lint / parse);
    println!("format:     {format:.3} s = {:.2} MB/s  ({changed} files changed) = {:.2} parses", mb / format, format / parse);
    per_file.retain(|f| f.2 >= 2000);
    for (label, key) in [("lint", 0), ("format", 1)] {
        let t = |f: &(f64, f64, usize, &PathBuf)| if key == 0 { f.0 } else { f.1 };
        per_file.sort_by(|a, b| (t(b) / b.2 as f64).total_cmp(&(t(a) / a.2 as f64)));
        println!("worst {label} ms per 10 KB (files >= 2 KB):");
        for f in per_file.iter().take(8) {
            let name = f.3.strip_prefix(dir).unwrap_or(f.3).display();
            println!("  {:7.2} ms/10KB {:8.2} ms {:6.1} KB  {name}", t(f) * 1e7 / f.2 as f64, t(f) * 1e3, f.2 as f64 / 1e3);
        }
    }
}

fn per_rule(clock: &Clock, reps: u32, codes: &[(PathBuf, Code)]) {
    let time = |providers: Vec<RuleV2Provider>| {
        let engine = engine(providers);
        codes.iter().fold((0.0, 0.0), |(l, f), (_, code)| {
            (
                l + clock.best_of(reps, || lint_once(&engine, code)),
                f + clock.best_of(reps, || format_once(&engine, code)),
            )
        })
    };
    let (base_lint, base_format) = time(vec![baseline()]);
    println!("baseline ({BASELINE_RULE}): lint {base_lint:.3} s, format {base_format:.3} s");
    let mut rows: Vec<(String, f64, f64)> = rule_providers_in(&standard_rule_providers(), KtlintVersion::V2_0)
        .into_iter()
        .map(|p| {
            let id = p.rule_id().value().to_owned();
            let (l, f) = time(vec![p]);
            eprintln!("{id}: {:.3} {:.3}", l - base_lint, f - base_format);
            (id, l - base_lint, f - base_format)
        })
        .collect();
    rows.sort_by(|a, b| (b.1 + b.2).total_cmp(&(a.1 + a.2)));
    println!("{:>9} {:>9}  rule (seconds over baseline)", "lint", "format");
    for (id, l, f) in rows {
        println!("{l:9.3} {f:9.3}  {id}");
    }
}

fn lint_once(engine: &KtLintRuleEngine, code: &Code) -> usize {
    let mut n = 0;
    let _ = engine.lint(code, &mut |_| n += 1);
    n
}

fn format_once(engine: &KtLintRuleEngine, code: &Code) -> Result<String, ktrs_lint::KtLintException> {
    engine.format(code, &mut |_| AutocorrectDecision::AllowAutocorrect)
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
