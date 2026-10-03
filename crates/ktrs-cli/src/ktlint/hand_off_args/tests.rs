use std::path::PathBuf;

use super::*;
use crate::ktlint::clikt::expand_argument_files;

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| a.to_string()).collect()
}

struct Dir(PathBuf);

impl Dir {
    fn new(tag: &str) -> Dir {
        let dir = std::env::temp_dir().join(format!("ktrs-hand-off-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Dir(dir)
    }

    fn write(&self, name: &str, text: &str) {
        std::fs::write(self.0.join(name), text).unwrap();
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn ktrs_option_is_removed_before_the_separator_only() {
    let dir = Dir::new("argv");
    let args = strings(&["-R", "x.jar", "--ktlint-version=1.8", "--ktlint-version", "2.0", "a.kt", "--", "--ktlint-version=1.8"]);
    let hand_off = hand_off_args(&args, &dir.0).unwrap();
    assert_eq!(hand_off.args, ["-R", "x.jar", "a.kt", "--", "--ktlint-version=1.8"]);
    assert!(!hand_off.has_temp_files());
}

#[test]
fn argfiles_without_the_option_are_passed_through() {
    let dir = Dir::new("plain");
    dir.write("args.txt", "-R x.jar src\n");
    let args = strings(&["@args.txt", "@@literal", "--ktlint-version", "1.8"]);
    let hand_off = hand_off_args(&args, &dir.0).unwrap();
    assert_eq!(hand_off.args, ["@args.txt", "@@literal"]);
    assert!(!hand_off.has_temp_files());
}

#[test]
fn option_in_an_argfile_is_dropped_and_the_rest_stays_in_an_argfile() {
    let dir = Dir::new("argfile");
    dir.write("inner.txt", "--ktlint-version=2.0 \"@@at\"\n");
    dir.write("args.txt", "# comment\n-R x.jar\n--ktlint-version 1.8\n'src/*.kt' \"C:\\\\a b\\\\\\\"q\\\".kt\" @inner.txt\n-- --ktlint-version=1.8\n");
    let args = strings(&["--relative", "@args.txt", "B.kt"]);
    let hand_off = hand_off_args(&args, &dir.0).unwrap();
    assert!(hand_off.has_temp_files());
    assert_eq!(hand_off.args.len(), 3);
    assert_eq!((hand_off.args[0].as_str(), hand_off.args[2].as_str()), ("--relative", "B.kt"));
    let temp = PathBuf::from(hand_off.args[1].strip_prefix('@').unwrap());
    assert!(temp.is_absolute() && temp.is_file());
    let reread = expand_argument_files(&hand_off.args, &dir.0).unwrap();
    assert_eq!(reread, ["--relative", "-R", "x.jar", "src/*.kt", "C:\\a b\\\"q\".kt", "@at", "--", "--ktlint-version=1.8", "B.kt"]);
    drop(hand_off);
    assert!(!temp.exists());
}

#[test]
fn value_of_the_option_may_come_from_the_next_argfile() {
    let dir = Dir::new("split");
    dir.write("value.txt", "1.8 a.kt\n");
    let args = strings(&["--ktlint-version", "@value.txt"]);
    let hand_off = hand_off_args(&args, &dir.0).unwrap();
    assert_eq!(expand_argument_files(&hand_off.args, &dir.0).unwrap(), ["a.kt"]);
}
