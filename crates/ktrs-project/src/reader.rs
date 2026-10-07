use std::path::{Path, PathBuf};

/// Reads build files, remembering each path asked for (present or not) so `invalidate` can match it.
#[derive(Default)]
pub(crate) struct Reader {
    inputs: Vec<PathBuf>,
}

impl Reader {
    pub(crate) fn read(&mut self, path: &Path) -> Option<String> {
        if !self.inputs.iter().any(|p| p == path) {
            self.inputs.push(path.to_path_buf());
        }
        let bytes = std::fs::read(path).ok()?;
        Some(String::from_utf8_lossy(&bytes).replace("\r\n", "\n"))
    }

    pub(crate) fn into_inputs(self) -> Vec<PathBuf> {
        self.inputs
    }
}

/// `path` relative to `root` with `/` separators, for notes.
pub(crate) fn display(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}
