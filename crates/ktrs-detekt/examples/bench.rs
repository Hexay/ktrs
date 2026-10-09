//! `cargo run -p ktrs-detekt --release --example bench [dir] [reps] [--per-rule]`: single-threaded cost of the
//! rule pass (the default config's active rules) next to the parse, per file best-of-`reps` wall time.
//! Compare runs by the "rule pass = N parses" line; `--per-rule` adds each rule's own pass (one more run each).

use std::path::PathBuf;
use std::time::Instant;
use std::{env, fs};

use ktrs_detekt::config::{load_configuration, workaround_configuration};
use ktrs_detekt::engine::{AnalysisMode, Analyzer, create_analyzer, get_rules};
use ktrs_detekt::kt_file::{load_text, parse_kt_file};
use ktrs_detekt::probe::kotlin_files;
use ktrs_detekt::rules::default_rule_set_providers;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn best_of<R>(reps: u32, mut f: impl FnMut() -> R) -> (f64, R) {
    let mut best: Option<(f64, R)> = None;
    for _ in 0..reps.max(1) {
        let started = Instant::now();
        let result = f();
        let seconds = started.elapsed().as_secs_f64();
        if best.as_ref().is_none_or(|(b, _)| seconds < *b) {
            best = Some((seconds, result));
        }
    }
    best.unwrap()
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let per_rule = args.iter().any(|a| a == "--per-rule");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let dir = PathBuf::from(positional.first().map_or("corpus", |s| s.as_str()));
    let reps: u32 = positional.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let base_path = fs::canonicalize(&dir).expect("the corpus directory exists");
    let config = workaround_configuration(load_configuration(&[]).expect("no config to load"), false, false, false);
    let analyzer = create_analyzer(base_path.clone(), &config);

    let sources: Vec<(PathBuf, String)> = kotlin_files(&base_path)
        .into_iter()
        .map(|path| {
            let text = load_text(&String::from_utf8_lossy(&fs::read(&path).unwrap_or_default()));
            (path, text)
        })
        .collect();
    let bytes: usize = sources.iter().map(|(_, text)| text.len()).sum();

    let (mut parse, mut rules, mut issues) = (0.0, 0.0, 0);
    for (path, text) in &sources {
        let name = path.to_string_lossy();
        let (parse_seconds, file) = best_of(reps, || parse_kt_file(text, &name));
        let (rule_seconds, found) = best_of(reps, || analyzer.analyze(&file, path).len());
        parse += parse_seconds;
        rules += rule_seconds;
        issues += found;
    }
    let mb = bytes as f64 / 1e6;
    println!("{} files, {mb:.1} MB, {} rules, {issues} issues", sources.len(), analyzer.rules().len());
    println!("parse:     {parse:.3} s = {:.1} MB/s", mb / parse);
    println!("rule pass: {rules:.3} s = {:.1} MB/s", mb / rules);
    println!("rule pass = {:.2} parses", rules / parse);

    if per_rule {
        let files: Vec<_> = sources.iter().map(|(path, text)| (path, parse_kt_file(text, &path.to_string_lossy()))).collect();
        let ids: Vec<String> = analyzer.rules().iter().map(|rule| rule.rule_instance.id.clone()).collect();
        let mut per_rule: Vec<(f64, String)> = ids
            .into_iter()
            .map(|id| {
                let all = get_rules(AnalysisMode::Light, &default_rule_set_providers(), &config, &mut |_| {});
                let only = all.into_iter().filter(|rule| rule.rule_instance.id == id).collect();
                let analyzer = Analyzer::new(base_path.clone(), only, AnalysisMode::Light);
                let started = Instant::now();
                files.iter().for_each(|(path, file)| drop(analyzer.analyze(file, path)));
                (started.elapsed().as_secs_f64(), id)
            })
            .collect();
        per_rule.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (seconds, id) in per_rule {
            println!("  {seconds:7.3} s = {:5.2} parses  {id}", seconds / parse);
        }
    }
}
