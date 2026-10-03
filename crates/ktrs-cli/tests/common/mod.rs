#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A fresh directory under the system temp dir, deleted on drop (upstream's `createTempDirectory`).
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ktrs_cli-{tag}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn write_text(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

pub fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|&a| a.to_owned()).collect()
}

pub const LS: &str = if cfg!(windows) { "\r\n" } else { "\n" };

pub struct Run {
    pub exit_code: i32,
    pub out: String,
    pub err: String,
}

/// `Main(input.byteInputStream(), PrintStream(out), PrintStream(err), args).run()`.
pub fn run(input: &str, args: &[&str]) -> Run {
    let main = ktrs_cli::ktfmt::Main::new(std::io::Cursor::new(input.as_bytes().to_vec()), Vec::new(), Vec::new());
    let exit_code = main.run(&strings(args));
    let (out, err) = main.into_streams();
    Run { exit_code, out: String::from_utf8(out).unwrap(), err: String::from_utf8(err).unwrap() }
}

/// `File.toString()`.
pub fn arg(path: &Path) -> String {
    path.display().to_string()
}

/// One `ktrs serve` frame.
pub fn frame(payload: &str) -> Vec<u8> {
    let mut bytes = (payload.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(payload.as_bytes());
    bytes
}

pub fn frames(mut bytes: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    while !bytes.is_empty() {
        let length = u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize;
        out.push(String::from_utf8(bytes[4..4 + length].to_vec()).unwrap());
        bytes = &bytes[4 + length..];
    }
    out
}

/// Runs a `ktrs serve` session over `requests`; the exit code and the response frames (hello first).
pub fn session(requests: &[&str]) -> (i32, Vec<String>) {
    let input: Vec<u8> = requests.iter().flat_map(|r| frame(r)).collect();
    let mut output = Vec::new();
    let code = ktrs_cli::serve::run(&input[..], &mut output);
    (code, frames(&output))
}

/// Truth's `containsExactly` without `inOrder()`.
pub fn assert_contains_exactly(mut actual: Vec<PathBuf>, expected: &[&Path]) {
    let mut expected: Vec<PathBuf> = expected.iter().map(|p| p.to_path_buf()).collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
}
