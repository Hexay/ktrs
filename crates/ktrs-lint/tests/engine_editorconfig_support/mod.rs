//! `KtlintTestFileSystem` stand-in: a fresh temp directory per instance (the `.editorconfig` cache is
//! global and keyed by path, so paths are never reused), removed on drop.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ktrs_lint::EditorConfig;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct TestFileSystem {
    pub root: PathBuf,
}

impl TestFileSystem {
    pub fn new() -> TestFileSystem {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("ktrs-lint-ec-{}-{nanos}-{n}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        TestFileSystem { root }
    }

    pub fn resolve(&self, relative: &str) -> PathBuf {
        if relative.is_empty() {
            self.root.clone()
        } else {
            self.root.join(relative)
        }
    }

    pub fn write_root_editor_config_file(&self, content: &str) {
        self.write_editor_config_file("", content);
    }

    pub fn write_editor_config_file(&self, relative_directory: &str, content: &str) {
        self.write_file(relative_directory, ".editorconfig", content);
    }

    pub fn write_file(&self, relative_directory: &str, file_name: &str, content: &str) {
        let dir = self.resolve(relative_directory);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(file_name), content).unwrap();
    }
}

impl Drop for TestFileSystem {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// `convertToPropertyValues()`: `name = value` lines.
pub fn convert_to_property_values(editor_config: &EditorConfig) -> Vec<String> {
    editor_config.map(|p| {
        let value = if p.is_unset() {
            "unset"
        } else {
            p.source_value().unwrap_or("null")
        };
        format!("{} = {value}", p.name())
    })
}

pub fn assert_contains(actual: &[String], expected: &[&str]) {
    for e in expected {
        assert!(
            actual.iter().any(|a| a == e),
            "expected {e:?} in {actual:?}"
        );
    }
}

pub fn assert_not_contains(actual: &[String], unexpected: &[&str]) {
    for e in unexpected {
        assert!(
            !actual.iter().any(|a| a == e),
            "unexpected {e:?} in {actual:?}"
        );
    }
}

pub fn assert_contains_exactly_in_any_order(actual: &[String], expected: &[&str]) {
    let mut a: Vec<&str> = actual.iter().map(String::as_str).collect();
    let mut e = expected.to_vec();
    a.sort_unstable();
    e.sort_unstable();
    assert_eq!(a, e);
}
