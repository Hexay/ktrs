//! Inputs for tools/fuzz/diff.sh: only files that parse without errors, since both upstream tools reject the rest.
//!
//!   kgen mutate SEEDS OUT COUNT [SEED] [MAX_BYTES]   1-4 splice/crossover edits of random seeds (.kt/.kts)
//!   kgen filter IN OUT [MAX_BYTES]                   a libFuzzer corpus dir -> its clean inputs, as OUT/<name>.kt
//!   kgen reduce FILE CMD...                          token-level delta debugging: shrinks FILE to FILE.min (same
//!                                                    extension) while `CMD... <candidate>` exits 0 (e.g.
//!                                                    tools/fuzz/differs.sh) and, if FILE did, it parses cleanly
//!   kgen dump FILE                                   the PSI dump (as tools/psi-dump/psi-dump.sh one) and whether
//!                                                    the tree spells the input

use std::collections::HashSet;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use ktrs_parser::FileKind;

const DEFAULT_MAX_BYTES: usize = 4000;
const ATTEMPTS_PER_OUTPUT: usize = 200;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |i: usize| args.get(i).map(String::as_str);
    let num = |i: usize, default: u64| arg(i).and_then(|s| s.parse().ok()).unwrap_or(default);
    match (arg(0), arg(1), arg(2)) {
        (Some("mutate"), Some(seeds), Some(out)) => {
            mutate(Path::new(seeds), Path::new(out), num(3, 1000) as usize, num(4, 1), num(5, DEFAULT_MAX_BYTES as u64) as usize)
        }
        (Some("filter"), Some(input), Some(out)) => filter(Path::new(input), Path::new(out), num(3, DEFAULT_MAX_BYTES as u64) as usize),
        (Some("reduce"), Some(file), Some(_)) => reduce(Path::new(file), &args[2..]),
        (Some("dump"), Some(file), _) => dump(Path::new(file)),
        _ => {
            eprintln!("usage: kgen mutate SEEDS OUT COUNT [SEED] [MAX_BYTES] | filter IN OUT [MAX_BYTES] | reduce FILE CMD... | dump FILE");
            std::process::exit(2);
        }
    }
}

struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn kind_of(path: &Path) -> FileKind {
    if path.extension().is_some_and(|e| e == "kts") { FileKind::Script } else { FileKind::Source }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn read_text(path: &Path, max_bytes: usize) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() > max_bytes {
        return None;
    }
    String::from_utf8(bytes).ok()
}

fn mutate(seeds_dir: &Path, out: &Path, count: usize, seed: u64, max_bytes: usize) {
    let mut paths = Vec::new();
    collect(seeds_dir, &mut paths);
    paths.retain(|p| p.extension().is_some_and(|e| e == "kt" || e == "kts"));
    paths.sort();
    let seeds: Vec<(String, FileKind)> =
        paths.iter().filter_map(|p| Some((read_text(p, max_bytes)?, kind_of(p)))).collect();
    assert!(!seeds.is_empty(), "no .kt/.kts seeds under {}", seeds_dir.display());
    fs::create_dir_all(out).unwrap();
    let mut rng = SplitMix(seed);
    let mut seen = HashSet::new();
    let (mut written, mut attempts) = (0, 0);
    while written < count && attempts < count * ATTEMPTS_PER_OUTPUT {
        attempts += 1;
        let (original, kind) = &seeds[rng.below(seeds.len())];
        let mut text = original.clone();
        for _ in 0..1 + rng.below(4) {
            let edited = if rng.below(4) == 0 {
                Some(ktrs_fuzz::crossover_text(&text, &seeds[rng.below(seeds.len())].0, rng.next()))
            } else {
                ktrs_fuzz::mutate(&text, rng.next())
            };
            text = edited.unwrap_or(text);
        }
        if text == *original || text.len() > max_bytes || !ktrs_fuzz::parses_cleanly(&text, *kind) || !seen.insert(hash(&text)) {
            continue;
        }
        let ext = if *kind == FileKind::Script { "kts" } else { "kt" };
        fs::write(out.join(format!("g{written:06}.{ext}")), &text).unwrap();
        written += 1;
    }
    println!("{} seeds, {attempts} attempts, {written} clean inputs written to {}", seeds.len(), out.display());
}

fn filter(input: &Path, out: &Path, max_bytes: usize) {
    let mut paths = Vec::new();
    collect(input, &mut paths);
    fs::create_dir_all(out).unwrap();
    let mut written = 0;
    for path in &paths {
        let Some(text) = read_text(path, max_bytes) else { continue };
        if text.trim().is_empty() || !ktrs_fuzz::parses_cleanly(&text, FileKind::Source) {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy();
        fs::write(out.join(format!("{name}.kt")), &text).unwrap();
        written += 1;
    }
    println!("{} inputs, {written} clean ones written to {}", paths.len(), out.display());
}

/// Greedy ddmin over token runs: tries deleting runs of `chunk` tokens, halving `chunk` down to one.
fn reduce(file: &Path, cmd: &[String]) {
    let kind = kind_of(file);
    let mut text = fs::read_to_string(file).unwrap();
    let ext = file.extension().map_or("kt".into(), |e| e.to_string_lossy());
    let candidate = file.with_extension(format!("cand.{ext}"));
    let keep_clean = ktrs_fuzz::parses_cleanly(&text, kind);
    let interesting = |t: &str| {
        fs::write(&candidate, t).unwrap();
        (!keep_clean || ktrs_fuzz::parses_cleanly(t, kind))
            && std::process::Command::new(&cmd[0]).args(&cmd[1..]).arg(&candidate).status().is_ok_and(|s| s.success())
    };
    assert!(interesting(&text), "{} is not interesting to begin with", file.display());
    let mut runs = 0;
    let mut chunk = ktrs_fuzz::token_boundaries(&text).len() / 2;
    while chunk >= 1 {
        let mut i = 0;
        loop {
            let b = ktrs_fuzz::token_boundaries(&text);
            if i + 1 >= b.len() {
                break;
            }
            let (s, e) = (b[i], b[(i + chunk).min(b.len() - 1)]);
            let shorter = [&text[..s], &text[e..]].concat();
            runs += 1;
            if interesting(&shorter) {
                text = shorter;
            } else {
                i += chunk;
            }
        }
        chunk /= 2;
    }
    let _ = fs::remove_file(&candidate);
    let out = file.with_extension(format!("min.{ext}"));
    fs::write(&out, &text).unwrap();
    println!("{runs} runs, {} bytes -> {}", text.len(), out.display());
}

fn dump(file: &Path) {
    let text = ktrs_fuzz::normalize_newlines(&fs::read_to_string(file).unwrap());
    let parse = ktrs_parser::parse_file(&text, kind_of(file));
    print!("{}", ktrs_syntax::psi_dump(&parse, &file.file_name().unwrap().to_string_lossy()));
    let tree = &parse.tree;
    let spelled: String = (0..tree.len() as u32).filter(|&e| tree.is_token(e)).map(|e| tree.text_of(e)).collect();
    println!("tree text == input: {}, tokens spell input: {}", tree.text() == text, spelled == text);
}

fn hash(text: &str) -> u64 {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}
