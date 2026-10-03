mod common;

use common::{TempDir, frame, frames, session, write_text};
use ktrs_cli::serve;

#[test]
fn hello_then_one_response_per_request_in_order() {
    let (code, responses) = session(&["\nfun  f( ) = 1\n", "style=google\n\nfun f() = 1\n"]);
    assert_eq!(code, 0);
    assert_eq!(responses[0], format!("ktrs-serve 2 {}", env!("CARGO_PKG_VERSION")));
    assert_eq!(responses[1], "status=ok\nchanged=true\n\nfun f() = 1\n");
    assert_eq!(responses[2], "status=ok\nchanged=false\n\nfun f() = 1\n");
}

#[test]
fn options_override_the_style() {
    let code = "fun f() {\n  val x = 1\n}\n";
    let (_, responses) = session(&[&format!("style=google\nblock-indent=4\n\n{code}")]);
    assert_eq!(responses[1], "status=ok\nchanged=true\n\nfun f() {\n    val x = 1\n}\n");
}

#[test]
fn unused_imports_are_kept_on_request() {
    let code = "import a.B\n\nfun f() = 1\n";
    let (_, responses) = session(&[&format!("\n{code}"), &format!("remove-unused-imports=false\n\n{code}")]);
    assert_eq!(responses[1], "status=ok\nchanged=true\n\nfun f() = 1\n");
    assert_eq!(responses[2], format!("status=ok\nchanged=false\n\n{code}"));
}

#[test]
fn errors_answer_the_request_and_the_server_carries_on() {
    let (code, responses) = session(&["path=src/A.kt\n\nfun f( {\n", "bogus=1\n\n", "no header end", "\nval x = 1\n"]);
    assert_eq!(code, 0);
    assert!(responses[1].starts_with("status=error\n\nsrc/A.kt:1:"), "{}", responses[1]);
    assert_eq!(responses[2], "status=error\n\nunknown request key 'bogus'");
    assert_eq!(responses[3], "status=error\n\nrequest has no empty line after its header");
    assert_eq!(responses[4], "status=ok\nchanged=false\n\nval x = 1\n");
}

#[test]
fn errors_without_a_path_are_ktfmts_message_alone() {
    let (_, responses) = session(&["\nfun f( {\n"]);
    assert!(responses[1].starts_with("status=error\n\n1:"), "{}", responses[1]);
}

#[test]
fn editorconfig_applies_at_the_path_when_asked() {
    let dir = TempDir::new("serve-editorconfig");
    write_text(&dir.path().join(".editorconfig"), "root = true\n[*.kt]\nindent_size = 8\n");
    let path = dir.path().join("A.kt");
    let code = "fun f() {\n  val x = 1\n}\n";
    let (_, responses) = session(&[
        &format!("path={}\neditorconfig=true\n\n{code}", path.display()),
        &format!("path={}\n\n{code}", path.display()),
    ]);
    assert_eq!(responses[1], "status=ok\nchanged=true\n\nfun f() {\n        val x = 1\n}\n");
    assert_eq!(responses[2], format!("status=ok\nchanged=false\n\n{code}"));
}

#[test]
fn a_truncated_frame_ends_the_server() {
    let mut input = frame("\nval x = 1\n");
    input.extend_from_slice(&100u32.to_be_bytes());
    input.extend_from_slice(b"short");
    let mut output = Vec::new();
    assert_eq!(serve::run(&input[..], &mut output), 2);
    assert_eq!(frames(&output).len(), 2);
}
