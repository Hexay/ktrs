//! Port of ktfmt's `cli/MainTest.kt` (v0.65), third part (from `--lines formats the selected file
//! statement`), in order: partial formatting.

mod common;

use std::fs;

use common::{TempDir, arg, read_text, run, write_text};

const CODE: &str = "fun untouched ( ) =   1

fun test() {
  val selected    =   2
  val adjacent    =   3
}
";
const SELECTED_FORMATTED: &str = "fun untouched ( ) =   1

fun test() {
  val selected = 2
  val adjacent    =   3
}
";

#[test]
fn lines_formats_the_selected_file_statement() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, CODE);

    let r = run("", &["--lines=4", &arg(&file)]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(read_text(&file), SELECTED_FORMATTED);
}

#[test]
fn lines_formats_the_selected_stdin_statement() {
    let r = run(CODE, &["--lines=4", "-"]);
    assert_eq!(r.exit_code, 0);
    assert_eq!(r.out, SELECTED_FORMATTED);
}

#[test]
fn lines_uses_stdin_name_for_editor_config_when_the_named_file_does_not_exist() {
    let root = TempDir::new("main");
    write_text(&root.path().join(".editorconfig"), "root = true\n[src/Generated.kt]\nindent_size = 4");
    let named_file = root.path().join("src/Generated.kt");
    let code = "fun test() {\n  val selected    =   2\n  val adjacent    =   3\n}\n";

    let r = run(code, &["--enable-editorconfig", &format!("--stdin-name={}", arg(&named_file)), "--lines=2", "-"]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(r.out, "fun test() {\n    val selected = 2\n  val adjacent    =   3\n}\n");
    assert!(!named_file.exists());
}

#[test]
fn lines_formats_the_selected_class_member_statement() {
    let code = "class Sample {
  fun untouched ( ) =   1

  fun test() {
    val selected    =   2
    val adjacent    =   3
  }
}
";
    let r = run(code, &["--lines=5", "-"]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(
        r.out,
        "class Sample {
  fun untouched ( ) =   1

  fun test() {
    val selected = 2
    val adjacent    =   3
  }
}
"
    );
}

#[test]
fn lines_applies_import_cleanup_after_selected_formatting() {
    let code = "import com.unused.Sample
import com.used.FooBarBaz as Baz
import com.used.bar

fun untouched ( ) =   1

fun test() {
  val selected    =   2
  Baz(bar)
}
";
    let r = run(code, &["--lines=8", "-"]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(
        r.out,
        "import com.used.FooBarBaz as Baz
import com.used.bar

fun untouched ( ) =   1

fun test() {
  val selected = 2
  Baz(bar)
}
"
    );
}

#[test]
fn lines_applies_multiline_string_cleanup_after_selected_formatting() {
    let code = "val indent =
    \"\"\"     \n         example
          of
            a

         multiline
           string
         \"\"\"
         .trimIndent()

fun untouched ( ) =   1

fun test() {
  val selected    =   2
}
";
    let r = run(code, &["--lines=15", "-"]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(
        r.out,
        "val indent =
    \"\"\"
    example
     of
       a

    multiline
      string
    \"\"\"
        .trimIndent()

fun untouched ( ) =   1

fun test() {
  val selected = 2
}
"
    );
}

#[test]
fn offset_and_length_format_the_selected_file_cursor_line() {
    let root = TempDir::new("main");
    let file = root.path().join("foo.kt");
    write_text(&file, CODE);

    let offset = format!("--offset={}", CODE.find("selected").unwrap());
    let r = run("", &[&offset, "--length=0", &arg(&file)]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(read_text(&file), SELECTED_FORMATTED);
}

#[test]
fn offset_and_length_format_the_selected_stdin_cursor_line() {
    let offset = format!("--offset={}", CODE.find("selected").unwrap());
    let r = run(CODE, &[&offset, "--length=0", "-"]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(r.out, SELECTED_FORMATTED);
}

#[test]
fn lines_rejects_directories_that_expand_to_multiple_files() {
    let root = TempDir::new("main");
    let dir = root.path().join("dir");
    fs::create_dir_all(&dir).unwrap();
    write_text(&dir.join("foo.kt"), "fun foo () = 1");
    write_text(&dir.join("bar.kt"), "fun bar () = 1");

    let r = run("", &["--lines=1", &arg(&dir)]);

    assert_eq!(r.exit_code, 1);
    assert!(r.err.contains("partial formatting is only supported for a single file"), "{}", r.err);
}

// Not upstream's: `--offset` counts UTF-16 units of the code with normalized line separators.
#[test]
fn offset_counts_utf16_units_after_line_separator_normalization() {
    let code = "val s = \"😀é\"\r\nfun test() {\r\n  val selected    =   2\r\n  val adjacent    =   3\r\n}\r\n";
    let normalized = code.replace("\r\n", "\n");
    let offset = normalized[..normalized.find("selected").unwrap()].encode_utf16().count();

    let r = run(code, &[&format!("--offset={offset}"), "--length=8", "-"]);

    assert_eq!(r.exit_code, 0);
    assert_eq!(r.out, "val s = \"😀é\"\r\nfun test() {\r\n  val selected = 2\r\n  val adjacent    =   3\r\n}\r\n");
}
