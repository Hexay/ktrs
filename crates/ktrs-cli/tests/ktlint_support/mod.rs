#![allow(dead_code)]
//! `CommandLineTestRunner` in process: a temp project dir with the given files, the `ktlint` command run
//! with it as working directory.

use std::fs;
use std::path::{Path, PathBuf};

use ktrs_cli::ktlint::KtlintCli;
use ktrs_cli::ktlint::console::Console;
use ktrs_cli::ktlint::jpath::JPath;
use ktrs_cli::ktlint::ktlint_jar::JvmEnv;

#[path = "../common/mod.rs"]
mod common;
pub use common::TempDir;

pub struct Project {
    temp: TempDir,
    dir: PathBuf,
    /// No `java` unless a test sets one, so a JVM hand-off fails fast.
    pub jvm: JvmEnv,
}

impl Project {
    /// `files`: (path relative to the project dir, content).
    pub fn new(name: &str, files: &[(&str, &str)]) -> Project {
        let temp = TempDir::new(&format!("ktlint-{name}"));
        let dir = temp.path().join(name);
        fs::create_dir_all(&dir).unwrap();
        let project = Project { temp, dir, jvm: JvmEnv { path: Some(Default::default()), ..JvmEnv::default() } };
        files.iter().for_each(|(path, content)| project.write(path, content));
        project
    }

    pub fn write(&self, path: &str, content: &str) {
        let file = self.dir.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, content).unwrap();
    }

    pub fn write_bytes(&self, path: &str, content: &[u8]) {
        let file = self.dir.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, content).unwrap();
    }

    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.dir.join(path)).unwrap()
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The temp dir the project dir is in (upstream's `tempDir`).
    pub fn temp_dir(&self) -> &Path {
        self.temp.path()
    }

    pub fn run(&self, args: &[&str]) -> Run {
        self.run_with_stdin(args, b"")
    }

    pub fn run_with_stdin(&self, args: &[&str], stdin: &[u8]) -> Run {
        self.run_cli(args, stdin, |cli, args| cli.run(args))
    }

    /// `command` (the `ktlint` command or another entry point) on a `KtlintCli` in this project.
    pub fn run_cli(&self, args: &[&str], stdin: &[u8], command: impl FnOnce(&KtlintCli, &[String]) -> i32) -> Run {
        let (console, out, err) = Console::capture(stdin);
        let cli = KtlintCli { console, working_dir: JPath::from_path(&self.dir), user_home: String::new(), jvm: self.jvm.clone() };
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let exit_code = command(&cli, &args);
        Run { exit_code, out: out.text(), err: err.text() }
    }
}

#[derive(Debug)]
pub struct Run {
    pub exit_code: i32,
    /// `normalOutput`: stdout, where ktlint also logs.
    pub out: String,
    /// `errorOutput`.
    pub err: String,
}

impl Run {
    pub fn assert_normal_exit_code(&self) -> &Self {
        assert_eq!(self.exit_code, 0, "{self:#?}");
        self
    }

    pub fn assert_error_exit_code(&self) -> &Self {
        assert_ne!(self.exit_code, 0, "{self:#?}");
        self
    }

    pub fn assert_error_output_is_empty(&self) -> &Self {
        assert_eq!(self.err, "", "{self:#?}");
        self
    }
}

/// `containsLineMatching(Regex)`: some line fully matches `pattern`.
pub fn contains_line_matching(text: &str, pattern: &str) -> bool {
    let regex = regex::Regex::new(&format!("^(?:{pattern})$")).unwrap();
    text.lines().any(|line| regex.is_match(line))
}

/// `containsLineMatching(String)`: some line contains `fragment`.
pub fn contains_line(text: &str, fragment: &str) -> bool {
    text.lines().any(|line| line.contains(fragment))
}

#[macro_export]
macro_rules! assert_line {
    ($text:expr, $pattern:expr) => {
        assert!($crate::ktlint_support::contains_line_matching(&$text, $pattern), "no line matching {:?} in:\n{}", $pattern, $text)
    };
}

#[macro_export]
macro_rules! assert_no_line {
    ($text:expr, $pattern:expr) => {
        assert!(!$crate::ktlint_support::contains_line_matching(&$text, $pattern), "a line matches {:?} in:\n{}", $pattern, $text)
    };
}
