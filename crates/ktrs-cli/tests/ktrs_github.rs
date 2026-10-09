//! The `github` reporter of `ktrs lint` and `ktrs fmt --check`: GitHub Actions workflow commands.

mod ktlint_support;

use std::io::{Cursor, Write};
use std::path::Path;

use ktlint_support::{Project, Run};
use ktrs_cli::github_annotations::{Annotations, UnformattedFiles};
use ktrs_cli::ktrs_fmt::{parse_fmt_args, run_with as run_fmt};
use ktrs_cli::ktrs_lint::run_with as run_lint;

const CODE: &str = "val  a = 1\n";

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| a.to_string()).collect()
}

fn lint(project: &Project, args: &[&str]) -> Run {
    project.run_cli(args, b"", |cli, args| {
        run_lint(cli, args).unwrap_or_else(|message| {
            cli.console.err(&message);
            2
        })
    })
}

#[test]
fn properties_and_message_are_escaped() {
    let annotations = Annotations::new(Path::new("/w/sub"), None);
    assert_eq!(
        annotations.error("src/A,b.kt", 3, Some(5), "standard:no-semi", "100% bad\r\nnext: line, here"),
        "::error file=src/A%2Cb.kt,line=3,col=5,title=standard%3Ano-semi::100%25 bad%0D%0Anext: line, here"
    );
    assert_eq!(annotations.error("A.kt", 1, None, "ktrs fmt", "x"), "::error file=A.kt,line=1,title=ktrs fmt::x");
}

#[test]
fn file_is_relative_to_the_workspace_when_inside_it() {
    let annotations = Annotations::new(Path::new("/w/sub"), Some(Path::new("/w")));
    assert_eq!(annotations.error("src/A.kt", 1, None, "t", "m"), "::error file=sub/src/A.kt,line=1,title=t::m");
    assert_eq!(annotations.error("/w/B.kt", 1, None, "t", "m"), "::error file=B.kt,line=1,title=t::m");
    assert_eq!(annotations.error("./B.kt", 1, None, "t", "m"), "::error file=sub/B.kt,line=1,title=t::m");
    assert_eq!(annotations.error("/other/C.kt", 1, None, "t", "m"), "::error file=/other/C.kt,line=1,title=t::m");
    let elsewhere = Annotations::new(Path::new("/elsewhere"), Some(Path::new("/w")));
    assert_eq!(elsewhere.error("src/A.kt", 1, None, "t", "m"), "::error file=src/A.kt,line=1,title=t::m");
}

#[test]
fn unformatted_files_become_annotations_line_by_line() {
    let mut out = Vec::new();
    let mut writer = UnformattedFiles::new(&mut out, Annotations::new(Path::new("/w"), None));
    writer.write_all(b"src/A.kt\r\nsrc/").unwrap();
    writer.write_all(b"B.kt\n").unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "::error file=src/A.kt,line=1,title=ktrs fmt::File is not formatted\n\
         ::error file=src/B.kt,line=1,title=ktrs fmt::File is not formatted\n"
    );
}

#[test]
fn lint_reporter_annotates_each_error() {
    let project = Project::new("github", &[("src/A.kt", CODE), ("B.kt", "val b = 1\n")]);
    let run = lint(&project, &["--reporter", "github"]);
    assert_eq!(run.exit_code, 1, "{run:?}");
    let annotations: Vec<&str> = run.out.lines().filter(|line| line.starts_with("::")).collect();
    assert_eq!(annotations.len(), 1, "{run:?}");
    assert_line!(annotations[0], r"::error file=src/A\.kt,line=1,col=\d+,title=standard%3Ano-multi-spaces::.+");
    assert!(!run.out.contains("Summary error count"), "{run:?}");
}

#[test]
fn lint_reporter_leaves_out_what_format_fixed() {
    let project = Project::new("github", &[("A.kt", CODE)]);
    let run = lint(&project, &["--format", "--reporter=github"]);
    assert_eq!((run.exit_code, run.out.as_str()), (0, ""), "{run:?}");
}

#[test]
fn lint_reporter_combines_with_others() {
    let project = Project::new("github", &[("A.kt", CODE)]);
    let run = lint(&project, &["--reporter=github", "--reporter=plain,output=report.txt"]);
    assert_line!(run.out, "::error file=A\\.kt,.*");
    assert!(project.read("report.txt").contains("(standard:no-multi-spaces)"));
}

#[test]
fn ktlint_drop_in_has_no_github_reporter() {
    let project = Project::new("github", &[("A.kt", CODE)]);
    let run = project.run(&["--reporter=github", "A.kt"]);
    assert_ne!(run.exit_code, 0, "{run:?}");
    let all = format!("{}{}", run.out, run.err);
    assert_line!(all, r#".*reporter "github" wasn't found \(available: baseline, checkstyle, format, html, json, plain, plain-summary, sarif\)"#);
    let unknown = lint(&project, &["--reporter=nope", "A.kt"]);
    assert_line!(format!("{}{}", unknown.out, unknown.err), r#".*reporter "nope" wasn't found \(available: .*sarif, github\)"#);
}

#[test]
fn fmt_check_annotates_unformatted_files() {
    let project = Project::new("github", &[("A.kt", CODE), ("B.kt", "val b = 1\n")]);
    let dir = project.dir().display().to_string();
    let args = parse_fmt_args(&strings(&["--check", "--reporter", "github", &dir])).unwrap();
    let mut out = Vec::new();
    let exit_code = run_fmt(&args, project.dir(), Cursor::new(Vec::new()), &mut out, Vec::new()).unwrap();
    assert_eq!(exit_code, 1);
    // The test's temp dir is not in a workspace, so the path stays as ktfmt prints it.
    let file = project.dir().join("A.kt").display().to_string().replace('\\', "/").replace(':', "%3A");
    assert_eq!(String::from_utf8(out).unwrap(), format!("::error file={file},line=1,title=ktrs fmt::File is not formatted\n"));
}
