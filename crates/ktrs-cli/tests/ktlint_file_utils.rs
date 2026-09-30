//! Port of ktlint-cli's `FileUtilsTest` (`fileSequence`) on a real temp dir instead of Jimfs.

mod common;

use std::fs;
use std::path::Path;

use common::TempDir;
use ktrs_cli::ktlint::console::Console;
use ktrs_cli::ktlint::file_utils::{DEFAULT_PATTERNS, file_sequence};
use ktrs_cli::ktlint::jpath::JPath;
use ktrs_cli::ktlint::logger::{Level, Logger};

const JAVA_FILE_ROOT_DIRECTORY: &str = "Root.java";
const KT_FILE_ROOT_DIRECTORY: &str = "Root.kt";
const KTS_FILE_ROOT_DIRECTORY: &str = "Root.kts";
const JAVA_FILE_IN_HIDDEN_DIRECTORY: &str = "project1/.hidden/Ignored.java";
const KT_FILE_IN_HIDDEN_DIRECTORY: &str = "project1/.hidden/Ignored.kt";
const KTS_FILE_IN_HIDDEN_DIRECTORY: &str = "project1/.hidden/Ignored.kts";
const JAVA_FILE_IN_PROJECT_ROOT_DIRECTORY: &str = "project1/ProjectRoot.java";
const KT_FILE_IN_PROJECT_ROOT_DIRECTORY: &str = "project1/ProjectRoot.kt";
const KTS_FILE_IN_PROJECT_ROOT_DIRECTORY: &str = "project1/ProjectRoot.kts";
const KT_FILE1_IN_PROJECT_SUB_DIRECTORY: &str = "project1/src/main/kotlin/One.kt";
const KT_FILE2_IN_PROJECT_SUB_DIRECTORY: &str = "project1/src/main/kotlin/example/Two.kt";
const KTS_FILE_IN_PROJECT_SUB_DIRECTORY: &str = "project1/src/scripts/Script.kts";
const JAVA_FILE_IN_PROJECT_SUB_DIRECTORY: &str = "project1/src/main/java/One.java";
const SOME_FILE_IN_OTHER_PROJECT_ROOT_DIRECTORY: &str = "other-project/SomeFile.txt";

struct Fs(TempDir);

impl Fs {
    fn new() -> Fs {
        let fs = Fs(TempDir::new("ktlint-file-utils"));
        for file in [
            JAVA_FILE_ROOT_DIRECTORY,
            KT_FILE_ROOT_DIRECTORY,
            KTS_FILE_ROOT_DIRECTORY,
            JAVA_FILE_IN_HIDDEN_DIRECTORY,
            KT_FILE_IN_HIDDEN_DIRECTORY,
            KTS_FILE_IN_HIDDEN_DIRECTORY,
            JAVA_FILE_IN_PROJECT_ROOT_DIRECTORY,
            KT_FILE_IN_PROJECT_ROOT_DIRECTORY,
            KTS_FILE_IN_PROJECT_ROOT_DIRECTORY,
            KTS_FILE_IN_PROJECT_SUB_DIRECTORY,
            KT_FILE1_IN_PROJECT_SUB_DIRECTORY,
            KT_FILE2_IN_PROJECT_SUB_DIRECTORY,
            JAVA_FILE_IN_PROJECT_SUB_DIRECTORY,
            SOME_FILE_IN_OTHER_PROJECT_ROOT_DIRECTORY,
        ] {
            fs.create_file(file);
        }
        // On Windows `Files.isHidden` is the DOS attribute, not the leading dot.
        #[cfg(windows)]
        std::process::Command::new("attrib").arg("+h").arg(fs.root().join("project1/.hidden")).status().unwrap();
        fs
    }

    fn root(&self) -> &Path {
        self.0.path()
    }

    fn create_file(&self, file: &str) {
        let path = self.root().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "// Not relevant for test").unwrap();
    }

    fn resolve(&self, path: &str) -> JPath {
        JPath::from_path(&self.root().join(path))
    }

    fn get_files(&self, patterns: &[&str], root_dir: Option<&str>) -> Vec<String> {
        self.get_files_with_home(patterns, root_dir, "")
    }

    fn get_files_with_home(&self, patterns: &[&str], root_dir: Option<&str>, user_home: &str) -> Vec<String> {
        let patterns: Vec<String> = patterns.iter().map(|p| p.to_string()).collect();
        let root_dir = self.resolve(root_dir.unwrap_or(""));
        let logger = Logger::new(Console::capture(b"").0, Level::Off);
        let base = JPath::from_path(self.root());
        let mut files: Vec<String> = file_sequence(&patterns, &root_dir, user_home, &logger)
            .unwrap()
            .iter()
            .map(|f| f.normalize().relative_to_or_self(&base).to_string())
            .collect();
        files.sort();
        files
    }
}

fn sorted(files: &[&str]) -> Vec<String> {
    let mut files: Vec<String> = files.iter().map(|f| f.to_string()).collect();
    files.sort();
    files
}

#[test]
fn given_no_patterns_and_no_workdir_then_find_all_kt_and_kts_files_except_in_hidden_directories() {
    let fs = Fs::new();
    let found = fs.get_files(&DEFAULT_PATTERNS, None);
    assert_eq!(
        found,
        sorted(&[
            KT_FILE_ROOT_DIRECTORY,
            KTS_FILE_ROOT_DIRECTORY,
            KT_FILE_IN_PROJECT_ROOT_DIRECTORY,
            KTS_FILE_IN_PROJECT_ROOT_DIRECTORY,
            KT_FILE1_IN_PROJECT_SUB_DIRECTORY,
            KT_FILE2_IN_PROJECT_SUB_DIRECTORY,
            KTS_FILE_IN_PROJECT_SUB_DIRECTORY,
        ])
    );
}

#[test]
fn given_some_patterns_and_no_workdir_then_ignore_all_files_in_hidden_directories() {
    let found = Fs::new().get_files(&["project1/**/*.kt", "project1/*.kt"], None);
    assert_eq!(found, sorted(&[KT_FILE_IN_PROJECT_ROOT_DIRECTORY, KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]));
}

#[test]
fn given_the_root_directory_where_scanning_starts_is_hidden_then_the_patterns_work() {
    let found = Fs::new().get_files(&["*.kt"], Some("project1/.hidden"));
    assert_eq!(found, sorted(&[KT_FILE_IN_HIDDEN_DIRECTORY]));
}

#[test]
fn given_patterns_including_a_negate_pattern_then_select_all_files_except_the_negated() {
    let found = Fs::new().get_files(&["project1/src/**/*.kt", "!project1/src/**/example/*.kt"], None);
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY]));
}

#[test]
fn given_a_pattern_and_a_workdir_then_find_the_matching_files_in_that_workdir() {
    let found = Fs::new().get_files(&["**/main/**/*.kt"], Some("project1"));
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]));
}

#[test]
fn given_a_pattern_containing_redundant_elements_then_they_are_ignored() {
    let fs = Fs::new();
    for pattern in [
        "./**/main/**/*.kt",
        "**/./main/**/*.kt",
        "**/main/./**/*.kt",
        "**/main/**/./*.kt",
        "xx/../**/main/**/./*.kt",
        "**/xx/../main/**/*.kt",
        "**/main/xx/../**/*.kt",
        "**/main/**/./xx/../*.kt",
    ] {
        let found = fs.get_files(&[pattern], Some("project1"));
        assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]), "pattern {pattern}");
    }
}

#[test]
fn given_a_relative_file_path_from_the_workdir_then_find_it() {
    let found = Fs::new().get_files(&["src/main/kotlin/One.kt"], Some("project1"));
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY]));
}

#[test]
fn given_an_absolute_file_path_and_a_workdir_then_find_both() {
    let fs = Fs::new();
    let absolute = fs.root().join(KT_FILE2_IN_PROJECT_SUB_DIRECTORY).display().to_string();
    let found = fs.get_files(&["src/main/kotlin/One.kt", &absolute], Some("project1"));
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]));
}

#[cfg(not(windows))]
#[test]
fn given_a_non_windows_os_and_a_pattern_starting_with_a_tilde_then_it_is_the_user_home() {
    // Upstream walks from the Jimfs root; here from the temp dir that holds the home directory.
    let fs = Fs::new();
    fs.create_file("home/project/src/main/kotlin/One.kt");
    let home = fs.root().join("home").display().to_string();
    for pattern in [
        "~/project/src/main/kotlin/One.kt",
        "~/project/src/main/kotlin/*.kt",
        "~/project/src/main/kotlin/",
        "~/project/src/main/kotlin",
        "~/project/src/main/**/*.kt",
    ] {
        let found = fs.get_files_with_home(&[pattern], None, &home);
        assert_eq!(found, sorted(&["home/project/src/main/kotlin/One.kt"]), "pattern {pattern}");
    }
}

#[test]
fn given_a_double_star_pattern_and_a_workdir_without_subdirectories_then_find_its_files() {
    let found = Fs::new().get_files(&["**/*.kt"], Some("project1/src/main/kotlin/"));
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]));
}

#[test]
fn given_a_pattern_with_multiple_double_stars_then_find_the_matching_files() {
    let found = Fs::new().get_files(&["src/**/kotlin/**/*.kt"], Some("project1"));
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]));
}

#[test]
fn given_a_relative_directory_path_then_find_its_files_with_the_default_kotlin_extensions() {
    let found = Fs::new().get_files(&["src/main/kotlin"], Some("project1"));
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]));
    assert!(!found.contains(&JAVA_FILE_IN_PROJECT_SUB_DIRECTORY.to_owned()));
}

#[cfg(windows)]
#[test]
fn given_windows_and_globs_using_backslash_then_they_are_converted_to_forward_slashes() {
    let found = Fs::new().get_files(&["project1\\src\\**\\*.kt", "!project1\\src\\**\\example\\*.kt"], None);
    assert_eq!(found, sorted(&[KT_FILE1_IN_PROJECT_SUB_DIRECTORY]));
}

#[cfg(not(windows))]
#[test]
fn on_non_windows_a_pattern_with_a_parent_directory_reference_may_leave_the_current_directory() {
    let fs = Fs::new();
    for pattern in ["../**/*.kt", "../**/src/main/kotlin/One.kt", "src/../../project1/src/**/*.kt", "src/../../project1/src/main/kotlin/*.kt"] {
        let found = fs.get_files(&[pattern], Some("project1"));
        assert!(found.contains(&KT_FILE1_IN_PROJECT_SUB_DIRECTORY.to_owned()), "pattern {pattern}: {found:?}");
    }
}

#[cfg(windows)]
#[test]
fn on_windows_a_pattern_with_a_parent_directory_reference_may_not_leave_the_current_directory() {
    let fs = Fs::new();
    for pattern in ["../**/*.kt", "../**/src/main/kotlin/One.kt", "src/../../project1/src/**/*.kt", "src/../../project1/src/main/kotlin/*.kt"] {
        let found = fs.get_files(&[pattern, "/some/non/existing/file"], Some("project1"));
        assert!(found.is_empty(), "pattern {pattern}: {found:?}");
    }
}

#[cfg(not(windows))]
#[test]
fn on_non_windows_a_wildcard_may_be_followed_by_a_parent_directory_reference() {
    let fs = Fs::new();
    for pattern in ["**/../**/*.kt", "**/../src/main/kotlin/One.kt", "src/main/k*/../kotlin/One.kt"] {
        let found = fs.get_files(&[pattern], Some("project1"));
        assert!(found.contains(&KT_FILE1_IN_PROJECT_SUB_DIRECTORY.to_owned()), "pattern {pattern}: {found:?}");
    }
}

#[cfg(windows)]
#[test]
fn on_windows_a_wildcard_may_not_be_followed_by_a_parent_directory_reference() {
    let fs = Fs::new();
    for pattern in ["**/../**/*.kt", "**/../src/main/kotlin/One.kt", "src/main/k*/../kotlin/One.kt"] {
        let found = fs.get_files(&[pattern, "/some/non/existing/file"], Some("project1"));
        assert!(found.is_empty(), "pattern {pattern}: {found:?}");
    }
}

#[test]
fn issue_1847_given_a_negate_pattern_only_then_include_the_default_patterns() {
    let found = Fs::new().get_files(&["!project1/**/*.kt"], None);
    assert_eq!(
        found,
        sorted(&[KT_FILE_ROOT_DIRECTORY, KTS_FILE_ROOT_DIRECTORY, KTS_FILE_IN_PROJECT_ROOT_DIRECTORY, KTS_FILE_IN_PROJECT_SUB_DIRECTORY])
    );
}

const PROJECT1_FILES: [&str; 5] = [
    KT_FILE_IN_PROJECT_ROOT_DIRECTORY,
    KTS_FILE_IN_PROJECT_ROOT_DIRECTORY,
    KT_FILE1_IN_PROJECT_SUB_DIRECTORY,
    KT_FILE2_IN_PROJECT_SUB_DIRECTORY,
    KTS_FILE_IN_PROJECT_SUB_DIRECTORY,
];

#[cfg(not(windows))]
#[test]
fn issue_2002_on_non_windows_find_files_in_a_sibling_directory_by_relative_path() {
    let found = Fs::new().get_files(&["../project1"], Some("other-project"));
    assert_eq!(found, sorted(&PROJECT1_FILES));
}

#[cfg(not(windows))]
#[test]
fn issue_2002_on_non_windows_find_files_in_a_sibling_directory_by_relative_glob() {
    let found = Fs::new().get_files(&["../project1/**/*.kt"], Some("other-project"));
    assert_eq!(found, sorted(&[KT_FILE_IN_PROJECT_ROOT_DIRECTORY, KT_FILE1_IN_PROJECT_SUB_DIRECTORY, KT_FILE2_IN_PROJECT_SUB_DIRECTORY]));
}

#[test]
fn issue_2002_find_files_in_a_sibling_directory_by_absolute_path() {
    let fs = Fs::new();
    let absolute = fs.root().join("project1").display().to_string();
    let found = fs.get_files(&[&absolute], Some("other-project"));
    assert_eq!(found, sorted(&PROJECT1_FILES));
}

#[test]
fn issue_2781_a_subdirectory_wildcard_before_a_trailing_double_star_does_not_match_the_file_name() {
    let fs = Fs::new();
    fs.create_file("project1/src/TestFoo.kt");
    fs.create_file("project1/src/Foo/FooTest.kt");
    let found = fs.get_files(&["**/*Foo*/**"], Some("project1"));
    assert_eq!(found, sorted(&["project1/src/Foo/FooTest.kt"]));
}
