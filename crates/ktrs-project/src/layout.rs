//! Which build owns a file: the nearest Gradle build file or `pom.xml`, and its root.

use std::path::{Path, PathBuf};

use crate::ProjectConfig;
use crate::reader::Reader;

pub(crate) const GRADLE_BUILD_FILES: [&str; 2] = ["build.gradle.kts", "build.gradle"];
pub(crate) const GRADLE_SETTINGS_FILES: [&str; 2] = ["settings.gradle.kts", "settings.gradle"];

pub(crate) enum Layout {
    /// `module`: the nearest build file at or below `root`, if any.
    Gradle { root: PathBuf, module: Option<PathBuf>, key: PathBuf },
    Maven { root: PathBuf, pom: PathBuf },
    None { dir: PathBuf },
}

pub(crate) fn locate(file: &Path) -> Layout {
    let start = if file.is_dir() { file } else { file.parent().unwrap_or(file) };
    let mut gradle_build: Option<PathBuf> = None;
    let mut settings_dir: Option<PathBuf> = None;
    let mut pom: Option<PathBuf> = None;
    let mut gradle_depth = usize::MAX;
    let mut pom_depth = usize::MAX;
    for (depth, dir) in start.ancestors().enumerate() {
        if gradle_build.is_none() {
            gradle_build = first_existing(dir, &GRADLE_BUILD_FILES);
            if gradle_build.is_some() {
                gradle_depth = gradle_depth.min(depth);
            }
        }
        if settings_dir.is_none() && first_existing(dir, &GRADLE_SETTINGS_FILES).is_some() {
            settings_dir = Some(dir.to_path_buf());
            gradle_depth = gradle_depth.min(depth);
        }
        if pom.is_none() && dir.join("pom.xml").is_file() {
            pom = Some(dir.join("pom.xml"));
            pom_depth = depth;
        }
        if settings_dir.is_some() && pom.is_some() {
            break;
        }
    }
    if let Some(pom) = pom.filter(|_| pom_depth < gradle_depth) {
        let mut root = pom.parent().unwrap_or(start).to_path_buf();
        while let Some(parent) = root.parent().filter(|p| p.join("pom.xml").is_file()) {
            root = parent.to_path_buf();
        }
        return Layout::Maven { root, pom };
    }
    if gradle_depth == usize::MAX {
        return Layout::None { dir: start.to_path_buf() };
    }
    let module_dir = gradle_build.as_deref().and_then(Path::parent);
    let root = match (&settings_dir, module_dir) {
        (Some(s), Some(m)) if m.starts_with(s) => s.clone(),
        (Some(s), None) => s.clone(),
        (_, Some(m)) => m.to_path_buf(),
        (None, None) => start.to_path_buf(),
    };
    let module = gradle_build.filter(|b| b.starts_with(&root));
    let key = module.clone().unwrap_or_else(|| root.clone());
    Layout::Gradle { root, module, key }
}

impl Layout {
    pub(crate) fn key(&self) -> &Path {
        match self {
            Layout::Gradle { key, .. } => key,
            Layout::Maven { pom, .. } => pom,
            Layout::None { dir } => dir,
        }
    }

    pub(crate) fn analyze(&self, script: bool, reader: &mut Reader) -> ProjectConfig {
        match self {
            Layout::Gradle { root, module, .. } => crate::gradle::analyze(root, module.as_deref(), script, reader),
            Layout::Maven { root, pom } => crate::maven::analyze(root, pom, reader),
            Layout::None { dir } => {
                ProjectConfig { root: dir.clone(), format: None, ktlint: None, notes: Vec::new() }
            }
        }
    }
}

pub(crate) fn first_existing(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    names.iter().map(|n| dir.join(n)).find(|p| p.is_file())
}
