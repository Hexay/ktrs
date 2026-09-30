//! Port of ktfmt's `cli/MainTest.kt` (v0.64), first half; the rest, in order, is `ktfmt_main_flags.rs`.

mod common;

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use common::{TempDir, arg, assert_contains_exactly, read_text, run, write_text};
use ktrs_cli::ktfmt::expand_args_to_file_names;

fn modified(path: &Path) -> SystemTime {
    fs::metadata(path).unwrap().modified().unwrap()
}

/// Scenario: someone _really_ wants to format this file, regardless of its extension. When a single
/// argument file is given, it is used as is without filtering by extension.
#[test]
fn expand_args_to_file_names_single_file_arg_is_used_as_is() {
    let root = TempDir::new("main");
    let foo_bar = root.path().join("foo.bar");
    write_text(&foo_bar, "hi");
    assert_contains_exactly(expand_args_to_file_names(&[arg(&foo_bar)]), &[&foo_bar]);
}

#[test]
fn expand_args_to_file_names_single_arg_which_is_not_a_file_is_not_returned() {
    let root = TempDir::new("main");
    let foo_bar = root.path().join("foo.bar");
    assert!(expand_args_to_file_names(&[arg(&foo_bar)]).is_empty());
}

#[test]
fn expand_args_to_file_names_single_arg_which_is_a_directory_is_resolved_to_its_recursively_contained_kt_files() {
    let root = TempDir::new("main");
    let dir = root.path().join("dir");
    fs::create_dir_all(&dir).unwrap();
    let foo = dir.join("foo.kt");
    write_text(&foo, "");
    let bar = dir.join("bar.kt");
    write_text(&bar, "");
    assert_contains_exactly(expand_args_to_file_names(&[arg(&dir)]), &[&foo, &bar]);
}

#[test]
fn expand_args_to_file_names_multiple_directory_args_are_resolved_to_their_recursively_contained_kt_files() {
    let root = TempDir::new("main");
    let dir1 = root.path().join("dir1");
    fs::create_dir_all(&dir1).unwrap();
    let foo1 = dir1.join("foo1.kt");
    write_text(&foo1, "");
    let bar1 = dir1.join("bar1.kt");
    write_text(&bar1, "");

    // Upstream never creates dir2 and puts foo2/bar2 in dir1; mirrored as is.
    let dir2 = root.path().join("dir2");
    fs::create_dir_all(&dir1).unwrap();
    let foo2 = dir1.join("foo2.kt");
    write_text(&foo2, "");
    let bar2 = dir1.join("bar2.kt");
    write_text(&bar2, "");

    assert_contains_exactly(expand_args_to_file_names(&[arg(&dir1), arg(&dir2)]), &[&foo1, &bar1, &foo2, &bar2]);
}

#[test]
fn using_dash_as_the_filename_formats_an_input_stream() {
    let r = run("fun    f1 (  ) :    Int =    0", &["-"]);
    assert_eq!(r.out, "fun f1(): Int = 0\n");
}

#[test]
fn parsing_errors_are_reported_stdin() {
    let r = run("fun    f1 (  ", &["-"]);
    assert_eq!(r.exit_code, 1);
    assert!(r.err.starts_with("<stdin>:1:14: error: "), "{}", r.err);
}

#[test]
fn parsing_errors_are_reported_stdin_name() {
    let r = run("fun    f1 (  ", &["--stdin-name=file/Foo.kt", "-"]);
    assert_eq!(r.exit_code, 1);
    assert!(r.err.starts_with("file/Foo.kt:1:14: error: "), "{}", r.err);
}

#[test]
fn parsing_errors_are_reported_file() {
    let root = TempDir::new("main");
    let foo_bar = root.path().join("foo.kt");
    write_text(&foo_bar, "fun    f1 (  ");
    let r = run("", &[&arg(&foo_bar)]);
    assert_eq!(r.exit_code, 1);
    assert!(r.err.contains("foo.kt:1:14: error: "), "{}", r.err);
}

#[test]
fn parsing_error_for_multiple_trailing_lambdas() {
    let root = TempDir::new("main");
    let foo_bar = root.path().join("foo.kt");
    write_text(&foo_bar, "val x = foo(bar { } { zap = 2 })");
    let r = run("", &[&arg(&foo_bar)]);
    assert_eq!(r.exit_code, 1);
    assert!(r.err.contains("foo.kt:1:21: error: Maximum one trailing lambda is allowed"), "{}", r.err);
}

// Upstream pins Main to a one-thread ForkJoinPool for serial processing; the assertions don't depend on order.
#[test]
fn all_files_in_args_are_processed_even_if_one_of_them_has_an_error() {
    let root = TempDir::new("main");
    let file1 = root.path().join("file1.kt");
    let file2_broken = root.path().join("file2.kt");
    let file3 = root.path().join("file3.kt");
    write_text(&file1, "fun    f1 ()  ");
    write_text(&file2_broken, "fun    f1 (  ");
    write_text(&file3, "fun    f1 ()  ");

    let r = run("", &[&arg(&file1), &arg(&file2_broken), &arg(&file3)]);

    assert_eq!(r.exit_code, 1);
    assert!(r.err.contains(&format!("Done formatting {}", arg(&file1))), "{}", r.err);
    assert!(r.err.contains("file2.kt:1:14: error: "), "{}", r.err);
    assert!(r.err.contains(&format!("Done formatting {}", arg(&file3))), "{}", r.err);
}

#[test]
fn file_is_not_modified_if_it_is_already_formatted() {
    let root = TempDir::new("main");
    let formatted_file = root.path().join("formatted_file.kt");
    write_text(&formatted_file, "fun f() = println(\"hello, world\")\n");

    let before = modified(&formatted_file);
    run("", &[&arg(&formatted_file)]);
    assert_eq!(before, modified(&formatted_file));
}

#[test]
fn file_is_modified_if_it_is_not_formatted() {
    let root = TempDir::new("main");
    let unformatted_file = root.path().join("unformatted_file.kt");
    write_text(&unformatted_file, "fun f() =   println(  \"hello, world\")\n");

    let before = modified(&unformatted_file);
    // The test may run under 1ms, and we need to make sure the new file timestamp will be different
    std::thread::sleep(std::time::Duration::from_millis(100));
    run("", &[&arg(&unformatted_file)]);
    assert!(before < modified(&unformatted_file));
}

#[test]
fn kotlinlang_style_is_passed_to_formatter_file() {
    let code = "fun f() {
    for (child in
        node.next.next.next.next.next.next.next.next.next.next.next.next.next.next.data()) {
        println(child)
    }
}
";
    let root = TempDir::new("main");
    let foo_bar = root.path().join("foo.kt");
    write_text(&foo_bar, code);

    run("", &["--kotlinlang-style", &arg(&foo_bar)]);

    assert_eq!(read_text(&foo_bar), code);
}

#[test]
fn kotlinlang_style_is_passed_to_formatter_stdin() {
    let code = "fun f() {
for (child in
node.next.next.next.next.next.next.next.next.next.next.next.next.next.next.data()) {
println(child)
}
}
";
    let formatted = "fun f() {
    for (child in
        node.next.next.next.next.next.next.next.next.next.next.next.next.next.next.data()) {
        println(child)
    }
}
";
    let r = run(code, &["--kotlinlang-style", "-"]);
    assert_eq!(r.out, formatted);
}
