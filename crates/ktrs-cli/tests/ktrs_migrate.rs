//! `ktrs migrate`: output, exit codes and `--write`, on copies of ktrs-project's fixtures.

use std::path::{Path, PathBuf};

use ktrs_cli::migrate::run;

/// Copies `crates/ktrs-project/tests/fixtures/<name>` to a fresh temp dir named `test` (tests run in parallel).
fn copy_fixture(name: &str, test: &str) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ktrs-project/tests/fixtures").join(name);
    let dst = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ktrs-migrate").join(test);
    let _ = std::fs::remove_dir_all(&dst);
    copy_dir(&src, &dst);
    dst
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap().flatten() {
        let (from, to) = (e.path(), dst.join(e.file_name()));
        if from.is_dir() {
            if e.file_name() != ".gradle" {
                copy_dir(&from, &to);
            }
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn migrate(args: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&args, &mut out, &mut err);
    (code, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

#[test]
fn dry_run_then_write_then_nothing() {
    let dir = copy_fixture("migrate-ktfmt-gradle", "dry_run");
    let path = dir.to_str().unwrap();
    let build = dir.join("build.gradle.kts");
    let before = std::fs::read_to_string(&build).unwrap();

    let (code, out, err) = migrate(&[path]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("-    id(\"com.ncorti.ktfmt.gradle\") version \"0.27.0\"\n"), "{out}");
    let version = env!("CARGO_PKG_VERSION");
    assert!(out.contains(&format!("+    id(\"io.github.hexay.ktrs\") version \"{version}\"\n")), "{out}");
    assert!(out.ends_with("1 file would change; run `ktrs migrate --write` to apply.\n"), "{out}");
    assert_eq!(err, "");
    assert_eq!(std::fs::read_to_string(&build).unwrap(), before, "a dry run writes nothing");

    let (code, out, _) = migrate(&["--write", path]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("migrated ") && out.contains("build.gradle.kts\n"), "{out}");
    assert!(out.ends_with("1 file changed.\n"), "{out}");
    assert!(std::fs::read_to_string(&build).unwrap().contains("id(\"io.github.hexay.ktrs\")"));

    let (code, out, _) = migrate(&[path]);
    assert_eq!((code, out.as_str()), (0, "Nothing to migrate.\n"));
}

#[test]
fn write_keeps_crlf() {
    let dir = copy_fixture("migrate-ktfmt-gradle", "crlf");
    let build = dir.join("build.gradle.kts");
    let crlf = std::fs::read_to_string(&build).unwrap().replace('\n', "\r\n");
    std::fs::write(&build, crlf).unwrap();
    assert_eq!(migrate(&["--write", dir.to_str().unwrap()]).0, 0);
    let after = std::fs::read_to_string(&build).unwrap();
    assert!(after.contains("io.github.hexay.ktrs") && !after.replace("\r\n", "").contains('\n'), "{after:?}");
}

#[test]
fn notes_go_to_stderr_and_dont_fail() {
    let dir = copy_fixture("kotlinter-groovy", "notes");
    let (code, out, err) = migrate(&[dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(out, "Nothing to migrate.\n");
    assert!(err.starts_with("note: build.gradle: kotlinter"), "{err}");
}

#[test]
fn paths_in_one_build_are_migrated_once() {
    let dir = copy_fixture("maven-gantsign", "paths");
    let (code, out, _) = migrate(&[dir.to_str().unwrap(), dir.join("module").to_str().unwrap()]);
    assert_eq!(code, 1);
    assert!(out.ends_with("2 files would change; run `ktrs migrate --write` to apply.\n"), "{out}");
}

#[test]
fn usage() {
    let (code, out, _) = migrate(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("Usage: ktrs migrate [--write] [PATH ...]"), "{out}");
    let (code, _, err) = migrate(&["--bogus"]);
    assert_eq!(code, 2);
    assert!(err.starts_with("error: unknown option --bogus"), "{err}");
}
