//! GitHub Actions workflow commands (`::error file=…,line=…,col=…,title=…::message`), which show a finding inline
//! on a pull request's diff: the `github` reporter of `ktrs lint` and of `ktrs fmt --check`.
//!
//! GitHub resolves `file` against the repository root, so a path under `GITHUB_WORKSPACE` is written relative
//! to it whatever the working directory; any other path stays as reported.

use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

pub const REPORTER_ID: &str = "github";

const NOT_FORMATTED_TITLE: &str = "ktrs fmt";
const NOT_FORMATTED_MESSAGE: &str = "File is not formatted";

#[derive(Clone, Debug)]
pub struct Annotations {
    working_dir: PathBuf,
    workspace: Option<PathBuf>,
}

impl Annotations {
    pub fn new(working_dir: &Path, workspace: Option<&Path>) -> Annotations {
        Annotations { working_dir: working_dir.to_path_buf(), workspace: workspace.map(Path::to_path_buf) }
    }

    /// With the runner's `GITHUB_WORKSPACE`.
    pub fn from_env(working_dir: &Path) -> Annotations {
        let workspace = std::env::var_os("GITHUB_WORKSPACE").filter(|w| !w.is_empty()).map(PathBuf::from);
        Annotations::new(working_dir, workspace.as_deref())
    }

    /// One `::error` command line (no line end); `col` is left out when `None`.
    pub fn error(&self, file: &str, line: usize, col: Option<usize>, title: &str, message: &str) -> String {
        let col = col.map(|col| format!(",col={col}")).unwrap_or_default();
        let (file, title) = (escape_property(&self.file(file)), escape_property(title));
        format!("::error file={file},line={line}{col},title={title}::{}", escape_data(message))
    }

    fn file(&self, file: &str) -> String {
        let absolute = self.working_dir.join(file);
        let path = self.workspace.as_ref().and_then(|w| absolute.strip_prefix(w).ok()).unwrap_or(Path::new(file));
        // `ktrs fmt .` names its files `./A.kt`.
        let path: PathBuf = path.components().filter(|c| !matches!(c, Component::CurDir)).collect();
        let path = path.to_string_lossy();
        if cfg!(windows) { path.replace('\\', "/") } else { path.into_owned() }
    }
}

/// The toolkit's `escapeData`.
fn escape_data(text: &str) -> String {
    text.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
}

/// The toolkit's `escapeProperty`.
fn escape_property(text: &str) -> String {
    escape_data(text).replace(':', "%3A").replace(',', "%2C")
}

/// Turns the output of a `ktfmt --dry-run` (a line per file that would change) into annotations on `out`.
pub struct UnformattedFiles<W> {
    out: W,
    annotations: Annotations,
    pending: Vec<u8>,
}

impl<W: Write> UnformattedFiles<W> {
    pub fn new(out: W, annotations: Annotations) -> UnformattedFiles<W> {
        UnformattedFiles { out, annotations, pending: Vec::new() }
    }
}

impl<W: Write> Write for UnformattedFiles<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(buf);
        while let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            let file = String::from_utf8_lossy(&line);
            let annotation = self.annotations.error(file.trim_end_matches(['\r', '\n']), 1, None, NOT_FORMATTED_TITLE, NOT_FORMATTED_MESSAGE);
            self.out.write_all(format!("{annotation}\n").as_bytes())?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}
