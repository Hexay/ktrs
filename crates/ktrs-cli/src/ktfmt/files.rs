//! `Main.kt`'s file handling: `expandArgsToFileNames`, and Java's `File` and `IOException` texts in its messages.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// `expandArgsToFileNames` expands `args` to a list of .kt files to format: a lone file as is,
/// otherwise every .kt/.kts file under each argument, in directory order (sorted here).
pub fn expand_args_to_file_names(args: &[String]) -> Vec<PathBuf> {
    if args.len() == 1 && Path::new(&args[0]).is_file() {
        return vec![PathBuf::from(&args[0])];
    }
    let mut result = Vec::new();
    for arg in args {
        walk_top_down(Path::new(arg), &mut result);
    }
    result
}

fn walk_top_down(path: &Path, result: &mut Vec<PathBuf>) {
    let Ok(metadata) = fs::metadata(path) else { return };
    if metadata.is_file() {
        // Kotlin's `File.extension`: whatever follows the name's last dot.
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        if matches!(name.rsplit_once('.'), Some((_, "kt" | "kts"))) {
            result.push(path.to_path_buf());
        }
    } else if metadata.is_dir() {
        let Ok(entries) = fs::read_dir(path) else { return };
        let mut children: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
        children.sort();
        for child in children {
            walk_top_down(&child, result);
        }
    }
}

/// `File.toString()`: Java normalizes separators to the platform's.
pub(super) fn java_file_name(file: &Path) -> String {
    let name = file.to_string_lossy();
    if cfg!(windows) { name.replace('/', "\\") } else { name.into_owned() }
}

/// A Java `IOException` message: `path (reason)` for a file, the OS reason alone otherwise.
pub(super) fn java_io_message(file: Option<&Path>, e: &io::Error) -> String {
    let message = e.to_string();
    let reason = message.split(" (os error").next().unwrap_or(&message).trim_end_matches('.');
    match file {
        Some(file) => format!("{} ({reason})", java_file_name(file)),
        None => reason.to_owned(),
    }
}
