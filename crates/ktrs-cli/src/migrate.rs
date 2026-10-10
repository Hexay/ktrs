//! `ktrs migrate`: the build-file rewrite planned by `ktrs_project::migrate`, shown as a diff or written.

use std::io::Write;
use std::path::{Path, PathBuf};

use ktrs_project::migrate::{Migration, plan};

pub const HELP: &str = "\
Usage: ktrs migrate [--write] [PATH ...]

Switches the build's ktfmt / ktlint setup to the ktrs drop-ins (default PATH: .): ktfmt-gradle,
ktlint-gradle and kotlinter plugin ids (plugins {}, the version catalog, buildscript or
convention-build dependencies) and the plugin repository, gantsign's ktlint-maven-plugin, and
Spotless's ktfmt/ktlint steps (Gradle and Maven). Edits only the ids, coordinates and versions, plus
the lines the README adds; setups with no drop-in (ktlint or ktfmt run from its jar) and settings a
drop-in doesn't take get a `note:` on stderr.

Without --write, prints the edits as a unified diff and exits 1 if there are any (0 if none).

Options:
  --write    Apply the edits and list the files changed";

pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let mut write = false;
    let mut paths = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--write" => write = true,
            "-h" | "--help" => {
                let _ = writeln!(out, "{HELP}");
                return 0;
            }
            _ if arg.starts_with('-') => {
                let _ = writeln!(err, "error: unknown option {arg}\n\n{HELP}");
                return 2;
            }
            _ => paths.push(PathBuf::from(arg)),
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    let mut migrations: Vec<Migration> = Vec::new();
    for path in &paths {
        let m = plan(path, env!("CARGO_PKG_VERSION"));
        if !migrations.iter().any(|seen| seen.root == m.root) {
            migrations.push(m);
        }
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut changed = 0;
    for m in &migrations {
        for note in &m.notes {
            let _ = writeln!(err, "note: {note}");
        }
        for change in &m.changes {
            let label = label(&cwd, &change.path);
            if write {
                if let Err(e) = change.write() {
                    let _ = writeln!(err, "error: {label}: {e}");
                    return 2;
                }
                let _ = writeln!(out, "migrated {label}");
            } else {
                let _ = write!(out, "{}", change.unified_diff(&label));
            }
            changed += 1;
        }
    }
    let files = if changed == 1 { "1 file".to_string() } else { format!("{changed} files") };
    match (changed, write) {
        (0, _) => {
            let _ = writeln!(out, "Nothing to migrate.");
            0
        }
        (_, true) => {
            let _ = writeln!(out, "{files} changed.");
            0
        }
        (_, false) => {
            let _ = writeln!(out, "{files} would change; run `ktrs migrate --write` to apply.");
            1
        }
    }
}

/// `path` relative to `cwd` when under it, with `/` separators.
fn label(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
