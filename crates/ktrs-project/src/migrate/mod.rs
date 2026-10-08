//! `ktrs migrate`: switches a build's ktfmt / ktlint setup to the ktrs drop-ins, mirroring README
//! "Integrations" (the swaps are listed in `coords.rs`). Edits are textual and in place: ids, coordinates and
//! version tokens are replaced, and the few lines the README adds (plugin repository, Spotless classpath and
//! Maven dependency) are inserted with the surrounding indentation. Anything that can't be rewritten without
//! guessing (versions from expressions, kotlinter, ktlint run from its jar, Spotless options with no ktrs
//! equivalent) becomes a note instead. Already migrated parts are left alone, so a second run is a no-op.

mod catalog_edits;
mod coords;
pub mod diff;
mod edits;
mod gradle;
mod gradle_plugins;
mod maven;
mod pom;
mod scan;
mod spotless_gradle;

use std::path::{Path, PathBuf};

use crate::layout::{self, Layout};
use crate::reader::display;
use coords::{README_DROP_IN, README_GRADLE, README_MAVEN};
use edits::Edits;

/// The ktlint versions the drop-ins run.
const VERSIONS_2_0_AND_1_8: [&str; 2] = ["1.8.0", "2.0.0-ALPHA-4"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    /// The build root (as in [`crate::ProjectConfig::root`]).
    pub root: PathBuf,
    pub changes: Vec<FileChange>,
    /// `<file relative to root>: <what to do by hand>; see README ...`.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: PathBuf,
    /// Contents with `\n` line ends; [`FileChange::write`] restores the file's CRLF.
    pub before: String,
    pub after: String,
    crlf: bool,
}

impl FileChange {
    pub fn write(&self) -> std::io::Result<()> {
        let text = if self.crlf { self.after.replace('\n', "\r\n") } else { self.after.clone() };
        std::fs::write(&self.path, text)
    }

    pub fn unified_diff(&self, label: &str) -> String {
        diff::unified(label, &self.before, &self.after)
    }
}

/// Plans the migration of the build containing `path` (a file or directory); writes nothing.
/// `ktrs_version` is what the swapped coordinates get.
pub fn plan(path: &Path, ktrs_version: &str) -> Migration {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut notes = Notes::default();
    let (root, docs) = match layout::locate(&path) {
        Layout::Gradle { root, .. } => {
            let docs = gradle::migrate(&root, ktrs_version, &mut notes);
            (root, docs)
        }
        Layout::Maven { root, .. } => {
            let mut docs = Vec::new();
            find_poms(&root, 0, &mut docs);
            let mut docs: Vec<Doc> = docs.iter().filter_map(|p| Doc::load(&root, p)).collect();
            maven::rewrite(&mut docs, ktrs_version, &mut notes);
            (root, docs)
        }
        Layout::None { dir } => {
            notes.file(".");
            notes.list.push(format!("{}: no Gradle or Maven build found", display(&dir, &path)));
            (dir, Vec::new())
        }
    };
    let changes = docs.into_iter().filter_map(Doc::finish).collect();
    Migration { root, changes, notes: notes.list }
}

fn find_poms(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if dir.join("pom.xml").is_file() {
        out.push(dir.join("pom.xml"));
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut dirs: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_dir()).collect();
    dirs.sort();
    for d in dirs {
        let name = d.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if depth < 12 && !matches!(name, "target" | ".git" | ".idea" | "node_modules" | "src" | ".mvn") {
            find_poms(&d, depth + 1, out);
        }
    }
}

fn rel(root: &Path, path: &Path) -> String {
    display(root, path)
}

/// A build file being rewritten: its LF-normalized text and the edits planned against it.
pub(crate) struct Doc {
    pub path: PathBuf,
    pub rel: String,
    pub text: String,
    pub edits: Edits,
    crlf: bool,
}

impl Doc {
    fn load(root: &Path, path: &Path) -> Option<Doc> {
        let raw = String::from_utf8(std::fs::read(path).ok()?).ok()?;
        let crlf = raw.contains("\r\n");
        let text = if crlf { raw.replace("\r\n", "\n") } else { raw };
        let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        Some(Doc { rel: rel(root, &path), path, text, edits: Edits::default(), crlf })
    }

    fn finish(self) -> Option<FileChange> {
        if self.edits.is_empty() {
            return None;
        }
        let after = self.edits.apply(&self.text);
        (after != self.text).then_some(FileChange { path: self.path, before: self.text, after, crlf: self.crlf })
    }
}

/// Notes for the file being processed, each pointing at the README section that explains the manual step.
#[derive(Default)]
pub(crate) struct Notes {
    file: String,
    list: Vec<String>,
}

impl Notes {
    pub(crate) fn file(&mut self, rel: &str) {
        rel.clone_into(&mut self.file);
    }

    fn add(&mut self, msg: String, readme: &str) {
        let note = format!("{}: {msg}; {readme}", self.file);
        if !self.list.contains(&note) {
            self.list.push(note);
        }
    }

    pub(crate) fn gradle(&mut self, msg: String) {
        self.add(msg, README_GRADLE);
    }

    pub(crate) fn maven(&mut self, msg: String) {
        self.add(msg, README_MAVEN);
    }

    pub(crate) fn drop_in(&mut self, msg: String) {
        self.add(msg, README_DROP_IN);
    }
}
