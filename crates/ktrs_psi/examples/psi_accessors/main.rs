//! Differential check of ktrs_psi against the compiler's PSI (tools/psi-accessors/psi-accessors.sh).
//!
//!   psi_accessors one <file> [--fixture] [--script]              print the report
//!   psi_accessors hashes <dir> [--fixture] [--script]            "<rel>\t<fnv1a64>" per file
//!   psi_accessors compare <dir> <jvm-hashes> [--fixture] [--script]  list files whose report differs
//!   psi_accessors dump <dir> <out-dir> [--fixture] [--script]    report per file into out-dir/<rel>.txt

mod report;

use std::path::Path;
use std::{env, fs, process, thread};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let fixture = args.iter().any(|a| a == "--fixture");
    let script = args.iter().any(|a| a == "--script");
    let positional: Vec<String> = args.into_iter().filter(|a| !a.starts_with("--")).collect();
    // Deep trees recurse through the visitor, like the JVM oracle (which runs with -Xss64m).
    let child = thread::Builder::new().stack_size(512 << 20).spawn(move || match positional
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["one", file] => print!("{}", report::report_file(Path::new(file), fixture, script)),
        ["hashes", dir] => {
            for (rel, hash) in report::hashes(Path::new(dir), fixture, script) {
                println!("{rel}\t{hash:016x}");
            }
        }
        ["compare", dir, expected] => compare(Path::new(dir), Path::new(expected), fixture, script),
        ["dump", dir, out] => {
            for (rel, path) in report::files(Path::new(dir), fixture) {
                let target = Path::new(out).join(format!("{rel}.txt"));
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::write(target, report::report_file(&path, fixture, script)).unwrap();
            }
        }
        _ => {
            eprintln!("usage: psi_accessors one <file> | hashes <dir> | compare <dir> <jvm-hashes> | dump <dir> <out>");
            process::exit(2);
        }
    });
    child.unwrap().join().unwrap();
}

fn compare(dir: &Path, expected: &Path, fixture: bool, script: bool) {
    let expected = fs::read_to_string(expected).unwrap();
    let actual = report::hashes(dir, fixture, script);
    let mismatched = report::mismatches(&expected, &actual);
    for rel in &mismatched {
        println!("DIFF {rel}");
    }
    println!("{}/{} files match", actual.len() - mismatched.len(), actual.len());
    if !mismatched.is_empty() {
        process::exit(1);
    }
}
