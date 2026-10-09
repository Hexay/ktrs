//! Java's `glob:` `PathMatcher` (`sun.nio.fs.Globs.toUnixRegexPattern`), matched against a path string.

use regex::Regex;

const REGEX_META_CHARS: &str = ".^$+{[]|()";
const GLOB_META_CHARS: &str = "\\*?[{";

fn is_regex_meta(c: char) -> bool {
    REGEX_META_CHARS.contains(c)
}

fn is_glob_meta(c: char) -> bool {
    GLOB_META_CHARS.contains(c)
}

/// `Globs.toRegexPattern(glob, isDos = false)`; `Err` is the `PatternSyntaxException` message.
pub fn to_unix_regex_pattern(glob: &str) -> Result<String, String> {
    let chars: Vec<char> = glob.chars().collect();
    let next = |i: usize| chars.get(i).copied();
    let mut in_group = false;
    let mut regex = String::from("^");
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        match c {
            '\\' => {
                let Some(next) = next(i) else { return Err("No character to escape".to_owned()) };
                if is_glob_meta(next) || is_regex_meta(next) {
                    regex.push('\\');
                }
                regex.push(next);
                i += 1;
            }
            '/' => regex.push(c),
            '[' => {
                regex.push_str("[[^/]&&[");
                if next(i) == Some('^') {
                    regex.push_str("\\^");
                    i += 1;
                } else {
                    if next(i) == Some('!') {
                        regex.push('^');
                        i += 1;
                    }
                    if next(i) == Some('-') {
                        regex.push('-');
                        i += 1;
                    }
                }
                let mut has_range_start = false;
                let mut last = '\0';
                let mut c = '\0';
                while i < chars.len() {
                    c = chars[i];
                    i += 1;
                    if c == ']' {
                        break;
                    }
                    if c == '/' {
                        return Err("Explicit 'name separator' in class".to_owned());
                    }
                    if c == '\\' || c == '[' || c == '&' && next(i) == Some('&') {
                        regex.push('\\');
                    }
                    regex.push(c);
                    if c == '-' {
                        if !has_range_start {
                            return Err("Invalid range".to_owned());
                        }
                        let end = next(i);
                        i += 1;
                        match end {
                            None | Some(']') => {
                                c = end.unwrap_or('\0');
                                break;
                            }
                            Some(end) if end < last => return Err("Invalid range".to_owned()),
                            Some(end) => {
                                c = end;
                                regex.push(end);
                            }
                        }
                        has_range_start = false;
                    } else {
                        has_range_start = true;
                        last = c;
                    }
                }
                if c != ']' {
                    return Err("Missing ']".to_owned());
                }
                regex.push_str("]]");
            }
            '{' => {
                if in_group {
                    return Err("Cannot nest groups".to_owned());
                }
                regex.push_str("(?:(?:");
                in_group = true;
            }
            '}' if in_group => {
                regex.push_str("))");
                in_group = false;
            }
            ',' if in_group => regex.push_str(")|(?:"),
            '*' if next(i) == Some('*') => {
                regex.push_str(".*");
                i += 1;
            }
            '*' => regex.push_str("[^/]*"),
            '?' => regex.push_str("[^/]"),
            c => {
                if is_regex_meta(c) {
                    regex.push('\\');
                }
                regex.push(c);
            }
        }
    }
    if in_group {
        return Err("Missing '}".to_owned());
    }
    regex.push('$');
    Ok(regex)
}

/// A compiled `glob:` matcher. Java regexes run in DOTALL-less mode, so `.` does not match line breaks.
#[derive(Clone, Debug)]
pub struct PathMatcher {
    glob: String,
    regex: Regex,
}

impl PathMatcher {
    pub fn new(glob: &str, case_insensitive: bool) -> Result<PathMatcher, String> {
        let pattern = to_unix_regex_pattern(glob)?;
        let pattern = if case_insensitive { format!("(?i){pattern}") } else { pattern };
        let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;
        Ok(PathMatcher { glob: glob.to_owned(), regex })
    }

    pub fn matches(&self, path: &str) -> bool {
        self.regex.is_match(path)
    }

    /// `toString()` of the JDK's matcher lambda is opaque; the glob is what a trace log can usefully show.
    pub fn glob(&self) -> &str {
        &self.glob
    }
}
