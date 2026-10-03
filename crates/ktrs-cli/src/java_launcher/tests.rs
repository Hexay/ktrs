use super::*;

fn split(command_line: &str) -> Vec<(String, bool)> {
    cmd_to_args(command_line)
}

#[test]
fn splits_like_the_launcher_and_flags_unquoted_wildcards() {
    let args = split(r#"ktlint.exe a "b c" src\*.kt "q/*.kt" x?y "#);
    let expected = [("ktlint.exe", false), ("a", false), ("b c", false), (r"src\*.kt", true), ("q/*.kt", false), ("x?y", true)];
    assert_eq!(args, expected.map(|(a, w)| (a.to_owned(), w)));
}

#[test]
fn backslashes_before_quotes_follow_the_launcher_rules() {
    let args = split(r#"p a\\\"b c\\"d e" f\g\"#);
    let expected = [("p", false), (r#"a\"b"#, false), (r"c\d e", false), (r"f\g\", false)];
    assert_eq!(args, expected.map(|(a, w)| (a.to_owned(), w)));
}

#[test]
fn passes_everything_as_is_when_the_splits_disagree() {
    let args = vec!["*.kt".to_owned()];
    assert_eq!(expand_application_args(args.clone(), "p *.kt extra"), args);
}

#[test]
fn quoted_or_unmatched_wildcards_stay() {
    let args = vec!["no-such-dir/*.kt".to_owned(), "*.kt".to_owned()];
    assert_eq!(expand_application_args(args.clone(), r#"p no-such-dir/*.kt "*.kt""#), args);
}

#[test]
fn file_splits_parent_and_name_like_java_io_file() {
    let cases = [
        ("C:/x//src/*.kt", Some(r"C:\x\src"), "*.kt"),
        (r"C:\*.kt", Some(r"C:\"), "*.kt"),
        ("*.kt", None, "*.kt"),
        ("./src/*.kt", Some(r".\src"), "*.kt"),
        (r"\*.kt", Some(r"\"), "*.kt"),
    ];
    for (arg, parent, name) in cases {
        let file = windows_path::File::new(arg);
        assert_eq!((file.parent().as_deref(), file.name().as_str()), (parent, name), "{arg}");
    }
}

#[test]
fn normalizes_entries_like_windows_path() {
    let norm = |s: &str, name: &str| windows_path::Path::parse(s).unwrap().resolve(name).normalize().to_string();
    assert_eq!(norm(".", "A.kt"), "A.kt");
    assert_eq!(norm(r".\src", "A.kt"), r"src\A.kt");
    assert_eq!(norm(r"C:\x\..\y", "A.kt"), r"C:\y\A.kt");
    assert_eq!(norm(r"..\x", "A.kt"), r"..\x\A.kt");
    assert!(windows_path::Path::parse("src*").is_none());
}

#[cfg(windows)]
#[test]
fn expands_against_the_directory_case_insensitively() {
    let dir = std::env::temp_dir().join(format!("ktrs-java-launcher-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["A.kt", "b.KT", "c.txt"] {
        std::fs::write(dir.join(f), "").unwrap();
    }
    let arg = format!("{}/*.kt", dir.display());
    let mut got = expand_application_args(vec![arg.clone()], &format!("p {arg}"));
    got.sort();
    let d = dir.display().to_string();
    assert_eq!(got, vec![format!(r"{d}\A.kt"), format!(r"{d}\b.KT")]);
    std::fs::remove_dir_all(&dir).unwrap();
}
