//! Port of ktfmt's `cli/MainTest.kt` (v0.65), second part (from `resolves 'kt' and 'kts'` to `--quiet`), in order.

mod common;

use std::fs;

use common::{LS, TempDir, arg, assert_contains_exactly, read_text, run, write_text};
use ktrs_cli::ktfmt::expand_args_to_file_names;

const UNFORMATTED: &str = "fun f () =    println( \"hello, world\" )";
const FORMATTED: &str = "fun f() = println(\"hello, world\")\n";

#[test]
fn expand_args_to_file_names_resolves_kt_and_kts_filenames_only_recursively() {
    let root = TempDir::new("main");
    let r = root.path();
    let (f1, f2, f3, f4, f5) = (r.join("1.kt"), r.join("2.kt"), r.join("3"), r.join("4.dummyext"), r.join("5.kts"));

    fs::create_dir_all(r.join("foo")).unwrap();
    let (f6, f7, f8) = (r.join("foo/1.kt"), r.join("foo/2.kts"), r.join("foo/3.dummyext"));
    let files = [&f1, &f2, &f3, &f4, &f5, &f6, &f7, &f8];
    for f in files {
        write_text(f, "");
    }
    let args: Vec<String> = files.iter().map(|f| arg(f)).collect();
    assert_contains_exactly(expand_args_to_file_names(&args), &[&f1, &f2, &f5, &f6, &f7]);
}

#[test]
fn formatting_from_stdin_prints_formatted_code_to_stdout_regardless_of_whether_it_was_already_formatted() {
    let r = run("fun f (   ) =    println(\"hello, world\")", &["-"]);
    assert_eq!(r.out, FORMATTED);

    let r = run("fun f () = println(\"hello, world\")", &["-"]);
    assert_eq!(r.out, FORMATTED);
}

#[test]
fn dry_run_prints_filename_and_does_not_change_file() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, UNFORMATTED);

    let r = run("", &["--dry-run", &arg(&file)]);

    assert_eq!(read_text(&file), UNFORMATTED);
    assert!(r.out.contains(&arg(&file)), "{}", r.out);
}

#[test]
fn dry_run_prints_stdin_and_does_not_reformat_code_from_stdin() {
    let r = run(UNFORMATTED, &["--dry-run", "-"]);
    assert!(!r.out.contains("hello, world"));
    assert_eq!(r.out, format!("<stdin>{LS}"));
}

// Upstream's code is a raw string, so `\n` is a literal backslash-n (a lexer error), not a newline.
#[test]
fn dry_run_prints_nothing_when_there_are_no_changes_needed_file() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, r#"fun f() = println("hello, world")\n"#);

    let r = run("", &["--dry-run", &arg(&file)]);

    assert_eq!(r.out, "");
}

#[test]
fn dry_run_prints_nothing_when_there_are_no_changes_needed_stdin() {
    let r = run(r#"fun f() = println("hello, world")\n"#, &["--dry-run", "-"]);
    assert_eq!(r.out, "");
}

#[test]
fn exit_code_is_0_when_there_are_changes_file() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, UNFORMATTED);
    assert_eq!(run("", &[&arg(&file)]).exit_code, 0);
}

#[test]
fn exits_with_0_when_there_are_changes_stdin() {
    assert_eq!(run(UNFORMATTED, &["-"]).exit_code, 0);
}

#[test]
fn exit_code_is_1_when_there_are_changes_and_set_exit_if_changed_is_set_file() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, UNFORMATTED);
    assert_eq!(run("", &["--set-exit-if-changed", &arg(&file)]).exit_code, 1);
}

#[test]
fn exit_code_is_1_when_there_are_changes_and_set_exit_if_changed_is_set_stdin() {
    assert_eq!(run(UNFORMATTED, &["--set-exit-if-changed", "-"]).exit_code, 1);
}

#[test]
fn set_exit_if_changed_and_dry_run_changes_nothing_prints_filenames_and_exits_with_1_file() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, UNFORMATTED);

    let r = run("", &["--dry-run", "--set-exit-if-changed", &arg(&file)]);

    assert_eq!(read_text(&file), UNFORMATTED);
    assert!(r.out.contains(&arg(&file)), "{}", r.out);
    assert_eq!(r.exit_code, 1);
}

#[test]
fn set_exit_if_changed_and_dry_run_changes_nothing_prints_filenames_and_exits_with_1_stdin() {
    let r = run(UNFORMATTED, &["--dry-run", "--set-exit-if-changed", "-"]);
    assert!(!r.out.contains("hello, world"));
    assert_eq!(r.out, format!("<stdin>{LS}"));
    assert_eq!(r.exit_code, 1);
}

// Upstream's stdout PrintStream is UTF-16; ours is a byte sink, so this checks the bytes are UTF-8.
#[test]
fn always_use_utf8_encoding_stdin_stdout() {
    let r = run(UNFORMATTED, &["-"]);
    assert_eq!(r.exit_code, 0);
    assert_eq!(r.out, FORMATTED);
}

#[test]
fn always_use_utf8_encoding_file() {
    let root = TempDir::new("main");
    let file = root.path().join("unformatted_file.kt");
    write_text(&file, "fun f() =   println(  \"hello, world\")\n");

    let r = run("", &[&arg(&file)]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(read_text(&file), FORMATTED);
}

#[test]
fn utf_8_bom_is_ignored_when_formatting_file() {
    let root = TempDir::new("main");
    let file = root.path().join("bom.kt");
    write_text(&file, "\u{feff}fun f () =    println( \"hello, world\" )");

    let r = run("", &[&arg(&file)]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(read_text(&file), FORMATTED);
}

#[test]
fn help_gives_return_code_of_0() {
    assert_eq!(run("", &["--help"]).exit_code, 0);
}

#[test]
fn quiet_suppresses_done_formatting_output() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, UNFORMATTED);

    let r = run("", &["--quiet", &arg(&file)]);

    assert!(!r.err.contains("Done formatting"), "{}", r.err);
}

#[test]
fn quiet_still_reports_errors() {
    let root = TempDir::new("main");
    let foo_bar = root.path().join("foo.kt");
    write_text(&foo_bar, "fun    f1 (  ");

    let r = run("", &["--quiet", &arg(&foo_bar)]);

    assert_eq!(r.exit_code, 1);
    assert!(r.err.contains("foo.kt:1:14: error: "), "{}", r.err);
}
