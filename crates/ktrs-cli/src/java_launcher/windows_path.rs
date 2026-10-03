//! The two Windows path flavours `LauncherHelper.expandArgs` goes through: `java.io.File` (`WinNTFileSystem`
//! normalize/prefix rules) for splitting the argument, and `java.nio` `WindowsPath` for the directory and its entries.

const SLASH: char = '\\';

/// `new File(path)` on Windows.
pub struct File {
    path: String,
    prefix_length: usize,
}

impl File {
    pub fn new(path: &str) -> File {
        let path = normalize_file_path(path);
        let prefix_length = prefix_length(&path);
        File { path, prefix_length }
    }

    /// `getParent()`.
    pub fn parent(&self) -> Option<String> {
        let index = self.path.rfind(SLASH);
        match index {
            Some(i) if i >= self.prefix_length => Some(self.path[..i].to_owned()),
            _ if self.prefix_length > 0 && self.path.len() > self.prefix_length => {
                Some(self.path[..self.prefix_length].to_owned())
            }
            _ => None,
        }
    }

    /// `getName()`.
    pub fn name(&self) -> String {
        match self.path.rfind(SLASH) {
            Some(i) if i >= self.prefix_length => self.path[i + 1..].to_owned(),
            _ => self.path[self.prefix_length..].to_owned(),
        }
    }
}

/// `WinNTFileSystem.normalize`: `/` to `\`, repeated separators collapsed (a leading UNC pair kept), no trailing
/// separator except on a root.
fn normalize_file_path(path: &str) -> String {
    let path = path.replace('/', "\\");
    let mut out = String::with_capacity(path.len());
    for (i, c) in path.chars().enumerate() {
        if c == SLASH && out.ends_with(SLASH) && !(i == 1 && out == "\\") {
            continue;
        }
        out.push(c);
    }
    if out.len() > 1 && out.ends_with(SLASH) && prefix_length(&out) != out.len() {
        out.pop();
    }
    out
}

/// `WinNTFileSystem.prefixLength`.
fn prefix_length(path: &str) -> usize {
    let b = path.as_bytes();
    match (b.first(), b.get(1)) {
        (None, _) => 0,
        (Some(b'\\'), Some(b'\\')) => 2,
        (Some(b'\\'), _) => 1,
        (Some(c), Some(b':')) if c.is_ascii_alphabetic() => {
            if b.get(2) == Some(&b'\\') { 3 } else { 2 }
        }
        _ => 0,
    }
}

/// A `java.nio` `WindowsPath`: a root (`C:\`, `C:`, `\`, `\\server\share\` or none) and names.
pub struct Path {
    root: String,
    names: Vec<String>,
}

impl Path {
    /// `Paths.get(s)`; `None` where `WindowsPathParser` throws `InvalidPathException`.
    pub fn parse(s: &str) -> Option<Path> {
        let s = s.replace('/', "\\");
        let (root, rest) = split_root(&s)?;
        let names: Vec<String> = rest.split(SLASH).filter(|n| !n.is_empty()).map(str::to_owned).collect();
        let invalid = |c: char| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*') || (c as u32) < 32;
        if names.iter().any(|n| n.chars().any(invalid) || n.ends_with(' ')) {
            return None;
        }
        Some(Path { root, names })
    }

    pub fn resolve(&self, name: &str) -> Path {
        let mut names = self.names.clone();
        names.push(name.to_owned());
        Path { root: self.root.clone(), names }
    }

    /// `normalize()`: drops `.`, folds `name\..`; a leading `..` stays on a relative path and goes on a rooted one.
    pub fn normalize(&self) -> Path {
        let mut names: Vec<String> = Vec::new();
        for name in &self.names {
            match name.as_str() {
                "." => {}
                ".." if names.last().is_some_and(|n| n != "..") => {
                    names.pop();
                }
                ".." if !self.root.is_empty() => {}
                _ => names.push(name.clone()),
            }
        }
        Path { root: self.root.clone(), names }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.root, self.names.join("\\"))
    }
}

fn split_root(s: &str) -> Option<(String, &str)> {
    let b = s.as_bytes();
    if s.starts_with("\\\\") {
        let mut parts = s[2..].splitn(3, SLASH);
        let (server, share) = (parts.next().filter(|p| !p.is_empty())?, parts.next().filter(|p| !p.is_empty())?);
        return Some((format!("\\\\{server}\\{share}\\"), parts.next().unwrap_or("")));
    }
    if b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' {
        let drive = &s[..2];
        return Some(match s[2..].strip_prefix(SLASH) {
            Some(rest) => (format!("{drive}\\"), rest),
            None => (drive.to_owned(), &s[2..]),
        });
    }
    Some(match s.strip_prefix(SLASH) {
        Some(rest) => ("\\".to_owned(), rest),
        None => (String::new(), s),
    })
}
