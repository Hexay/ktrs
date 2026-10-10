//! `ktrs ktlint --ktrs-kotlinter-events`: the kotlinter drop-in's hidden option (`crates/ktrs-cli/src/ktlint/kotlinter.rs`).

mod common;

use std::path::Path;

use common::{TempDir, read_text, strings, write_text};
use ktrs_cli::ktlint::KtlintCli;
use ktrs_cli::ktlint::jpath::JPath;

/// Runs in `dir` (kotlinter's project directory): report paths are relative to it.
fn ktlint(dir: &Path, args: &[&str]) -> i32 {
    let cli = KtlintCli { working_dir: JPath::from_path(dir), ..KtlintCli::from_env() };
    let mut all = vec!["--ktlint-version=1.8"];
    all.extend_from_slice(args);
    cli.run(&strings(&all))
}

fn path(p: &Path) -> String {
    p.display().to_string()
}

/// The events file's lines without the header, the directory's path replaced by `<D>`.
fn events(dir: &Path, file: &Path) -> Vec<String> {
    let prefix = format!("{}{}", path(dir), std::path::MAIN_SEPARATOR);
    read_text(file).lines().skip(1).map(|l| l.replace(&prefix, "<D>/")).collect()
}

#[test]
fn lint_events_keep_every_error_and_reports_drop_the_second_at_a_position() {
    let dir = TempDir::new("ktrs-kotlinter-lint");
    let wrong = dir.path().join("Wrong.kt");
    let clean = dir.path().join("Clean.kt");
    write_text(&wrong, "class Wrong{ }\n");
    write_text(&clean, "package com.example\n\nval foo = 1\n");
    let (out, plain, sarif) = (dir.path().join("events.txt"), dir.path().join("plain.txt"), dir.path().join("report.sarif.json"));
    let code = ktlint(
        dir.path(),
        &[
            &format!("--ktrs-kotlinter-events={}", path(&out)),
            &format!("--reporter=plain,output={}", path(&plain)),
            &format!("--reporter=sarif,output={}", path(&sarif)),
            &path(&wrong),
            &path(&clean),
        ],
    );
    assert_eq!(code, 1);
    let rows = events(dir.path(), &out);
    assert_eq!(rows.len(), 5, "{rows:?}");
    assert_eq!(rows[0], "file\t<D>/Wrong.kt");
    assert!(rows[1..4].iter().all(|r| r.starts_with("error\t<D>/Wrong.kt\t1\t12\tstandard:")), "{rows:?}");
    assert_ne!(rows[1], rows[2]);
    assert_eq!(rows[4], "file\t<D>/Clean.kt");
    let plain = read_text(&plain);
    assert_eq!(plain.lines().filter(|l| l.starts_with("Wrong.kt:1:12: ")).count(), 1, "relative path, one error per position: {plain}");
    let first_rule = rows[1].split('\t').nth(4).unwrap();
    assert!(plain.contains(&format!("({first_rule})")), "the first error in engine order is kept: {plain}");
    assert!(read_text(&sarif).contains("Wrong.kt"));
}

#[test]
fn format_events_follow_the_emit_order_and_name_the_rewritten_files() {
    let dir = TempDir::new("ktrs-kotlinter-format");
    let file = dir.path().join("A.kt");
    let clean = dir.path().join("Clean.kt");
    write_text(&file, "import java.util.*\n\nval  foo = 1\n");
    write_text(&clean, "val foo = 1\n");
    let out = dir.path().join("events.txt");
    let code = ktlint(dir.path(), &["--format", &format!("--ktrs-kotlinter-events={}", path(&out)), &path(&file), &path(&clean)]);
    assert_eq!(code, 1, "the wildcard import can't be fixed");
    assert_eq!(read_text(&file), "import java.util.*\n\nval foo = 1\n");
    let rows = events(dir.path(), &out);
    let fixed = "error\t<D>/A.kt\t3\t5\tstandard:no-multi-spaces\tLINT_CAN_BE_AUTOCORRECTED\tfalse\tUnnecessary long whitespace";
    let unfixable = "error\t<D>/A.kt\t1\t1\tstandard:no-wildcard-imports\tLINT_CAN_NOT_BE_AUTOCORRECTED\tfalse\tWildcard import";
    assert_eq!(rows.iter().filter(|r| *r == fixed).count(), 1, "{rows:?}");
    assert_eq!(rows.iter().filter(|r| *r == unfixable).count(), 2, "reported again by the pass after the fix: {rows:?}");
    assert_eq!(rows[0], "file\t<D>/A.kt");
    assert_eq!(rows[rows.len() - 2..], ["formatted\t<D>/A.kt", "file\t<D>/Clean.kt"]);
}

#[test]
fn a_parse_error_stops_the_run_at_its_file() {
    for format in [false, true] {
        let dir = TempDir::new("ktrs-kotlinter-parse-error");
        let (a, b, c) = (dir.path().join("A.kt"), dir.path().join("B.kt"), dir.path().join("C.kt"));
        write_text(&a, "val  a = 1\n");
        write_text(&b, "fun b( = \n");
        write_text(&c, "val  c = 1\n");
        let (out, plain) = (dir.path().join("events.txt"), dir.path().join("plain.txt"));
        let events_option = format!("--ktrs-kotlinter-events={}", path(&out));
        let reporter = format!("--reporter=plain,output={}", path(&plain));
        let mut args = vec![events_option.as_str(), reporter.as_str()];
        if format {
            args.push("--format");
        }
        let (a_arg, b_arg, c_arg) = (path(&a), path(&b), path(&c));
        args.extend([a_arg.as_str(), b_arg.as_str(), c_arg.as_str()]);
        assert_eq!(ktlint(dir.path(), &args), 1);
        let rows = events(dir.path(), &out);
        let last = rows.last().unwrap();
        assert!(last.starts_with("error\t<D>/B.kt\t") && last.contains("\t\tKOTLIN_PARSE_EXCEPTION\tfalse\t1:"), "{rows:?}");
        assert!(!rows.iter().any(|r| r.contains("C.kt")), "{rows:?}");
        assert_eq!(read_text(&a), if format { "val a = 1\n" } else { "val  a = 1\n" });
        assert_eq!(read_text(&c), "val  c = 1\n", "files after the exception are left alone");
        if !format {
            assert_eq!(read_text(&plain), "", "the reporters never get afterAll");
        }
    }
}
