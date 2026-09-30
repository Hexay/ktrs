//! Port of ec4j `model/Glob.java`: a section glob compiled to a regular expression. The generated
//! syntax is the Java one; the constructs it uses mean the same in the `regex` crate, except that `\d`
//! is written `[0-9]` (Java's is ASCII-only) and matching is anchored explicitly (`Matcher.matches`).

use regex::Regex;

#[derive(Clone, Debug)]
pub struct Glob {
    error: Option<String>,
    ranges: Vec<[i32; 2]>,
    regex: Option<Regex>,
    source: String,
    match_last_segment_only: bool,
}

impl PartialEq for Glob {
    fn eq(&self, other: &Glob) -> bool {
        self.source == other.source
    }
}

impl Eq for Glob {}

impl Glob {
    pub fn new(source: &str) -> Glob {
        let mut ranges = Vec::new();
        let unescaped = unescape_comment_signs(source);
        let slash_pos = unescaped.find('/');
        let double_asterisk_pos = unescaped.find("**");
        let pattern = match slash_pos {
            Some(0) => &unescaped[1..],
            _ => &unescaped[..],
        };
        let match_last_segment_only = slash_pos.is_none() && double_asterisk_pos.is_none();
        let mut regex = String::with_capacity(pattern.len());
        convert_glob_to_reg_ex(pattern, &mut ranges, &mut regex);
        let (regex, error) = match Regex::new(&format!(r"\A(?:{regex})\z")) {
            Ok(regex) => (Some(regex), None),
            Err(e) => (None, Some(e.to_string())),
        };
        Glob {
            error,
            ranges,
            regex,
            source: source.to_owned(),
            match_last_segment_only,
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        self.error.is_none()
    }

    /// `match(filePath)`: `file_path` is `/`-separated and relative to the `.editorconfig`'s directory.
    pub fn is_match(&self, file_path: &str) -> bool {
        let Some(regex) = &self.regex else {
            return false;
        };
        let subject = if self.match_last_segment_only {
            file_path.rsplit('/').next().unwrap_or(file_path)
        } else {
            file_path
        };
        let Some(captures) = regex.captures(subject) else {
            return false;
        };
        for (i, range) in self.ranges.iter().enumerate() {
            let Some(number_string) = captures.get(i + 1).map(|m| m.as_str()) else {
                return false;
            };
            if number_string.starts_with('0') {
                return false;
            }
            match number_string.parse::<i32>() {
                Ok(number) if number >= range[0] && number <= range[1] => {}
                // Integer.parseInt overflows with a NumberFormatException out of match().
                Err(_) => panic!("NumberFormatException: For input string: \"{number_string}\""),
                Ok(_) => return false,
            }
        }
        true
    }
}

impl std::fmt::Display for Glob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.source)
    }
}

/// `ESCAPED_COMMENT_SIGNS.matcher(source).replaceAll("$1")`: `\#` -> `#`, `\;` -> `;`.
fn unescape_comment_signs(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && matches!(chars.peek(), Some('#' | ';')) {
            continue;
        }
        out.push(c);
    }
    out
}

fn convert_glob_to_reg_ex(glob_string: &str, ranges: &mut Vec<[i32; 2]>, result: &mut String) {
    let glob: Vec<char> = glob_string.chars().collect();
    let length = glob.len();
    let mut i = 0;
    let mut brace_level = 0;
    let matching_braces = matching_braces(&glob);
    let mut escaped = false;
    let mut in_brackets = false;
    while i < length {
        let current = glob[i];
        i += 1;
        if current == '*' {
            if i < length && glob[i] == '*' {
                result.push_str(".*");
                i += 1;
            } else {
                result.push_str("[^/]*");
            }
        } else if current == '?' {
            result.push('.');
        } else if current == '[' {
            let seen_slash = find_char('/', ']', &glob, length, i) >= 0;
            if !seen_slash && !escaped {
                if i < length && "!^".contains(glob[i]) {
                    i += 1;
                    result.push_str("[^");
                } else {
                    result.push('[');
                }
            } else {
                result.push_str("\\[");
            }
            in_brackets = true;
        } else if current == ']' || current == '-' && in_brackets {
            if escaped {
                result.push('\\');
            }
            result.push(current);
            in_brackets = current != ']' || escaped;
        } else if current == '{' {
            let j = find_char(',', '}', &glob, length, i);
            if j < 0 && ((-j) as usize) < length {
                let choice: String = glob[i..(-j) as usize].iter().collect();
                match get_numeric_range(&choice) {
                    Some(range) => {
                        result.push_str("([0-9]+)");
                        ranges.push(range);
                    }
                    None => {
                        result.push_str("\\{");
                        convert_glob_to_reg_ex(&choice, ranges, result);
                        result.push_str("\\}");
                    }
                }
                i = (-j) as usize + 1;
            } else if matching_braces {
                result.push_str("(?:");
                brace_level += 1;
            } else {
                result.push_str("\\{");
            }
        } else if current == ',' {
            result.push_str(if brace_level > 0 && !escaped {
                "|"
            } else {
                ","
            });
        } else if current == '/' {
            if i < length
                && glob[i] == '*'
                && i + 2 < length
                && glob[i + 1] == '*'
                && glob[i + 2] == '/'
            {
                result.push_str("(?:/|/.*/)");
                i += 3;
            } else {
                result.push(current);
            }
        } else if current == '}' {
            if brace_level > 0 && !escaped {
                result.push(')');
                brace_level -= 1;
            } else {
                result.push('}');
            }
        } else if current != '\\' {
            escape_to_regex(current, result);
        }
        if current == '\\' {
            if escaped {
                result.push_str("\\\\");
            }
            escaped = !escaped;
        } else {
            escaped = false;
        }
    }
}

fn matching_braces(glob: &[char]) -> bool {
    let mut i = 0;
    let mut opened_count = 0;
    while i < glob.len() {
        let c = glob[i];
        i += 1;
        match c {
            '\\' => i += 1,
            '{' => opened_count += 1,
            '}' => opened_count -= 1,
            _ => {}
        }
    }
    opened_count == 0
}

fn get_numeric_range(choice: &str) -> Option<[i32; 2]> {
    let separator = choice.find("..")?;
    let start = choice[..separator].parse::<i32>().ok()?;
    let end = choice[separator + 2..].parse::<i32>().ok()?;
    Some([start, end])
}

/// The index of `c` before the first unescaped `stop_at`, or minus the index where the search ended.
fn find_char(c: char, stop_at: char, pattern: &[char], length: usize, start: usize) -> isize {
    let mut j = start;
    let mut escaped_char = false;
    while j < length && (pattern[j] != stop_at || escaped_char) {
        if pattern[j] == c && !escaped_char {
            return j as isize;
        }
        escaped_char = pattern[j] == '\\' && !escaped_char;
        j += 1;
    }
    -(j as isize)
}

fn escape_to_regex(c: char, result: &mut String) {
    if c == ' ' || c.is_alphabetic() || c.is_numeric() || c == '_' || c == '-' {
        result.push(c);
    } else if c == '\n' {
        result.push_str("\\n");
    } else if c.is_ascii() {
        result.push('\\');
        result.push(c);
    } else {
        // Java escapes any char with `\`; the regex crate accepts escapes of ASCII punctuation only.
        result.push_str(&regex::escape(&c.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::Glob;

    #[test]
    fn globs_like_ec4j() {
        assert!(Glob::new("*.kt").is_match("src/main/Foo.kt"));
        assert!(!Glob::new("*.kt").is_match("src/main/Foo.kts"));
        assert!(Glob::new("*.{kt,kts}").is_match("a/b.kts"));
        assert!(Glob::new("src/**/*.kt").is_match("src/a/b/C.kt"));
        assert!(!Glob::new("src/*.kt").is_match("src/a/C.kt"));
        assert!(Glob::new("/src/*.kt").is_match("src/C.kt"));
        assert!(Glob::new("file{1..3}.kt").is_match("file2.kt"));
        assert!(!Glob::new("file{1..3}.kt").is_match("file4.kt"));
        assert!(!Glob::new("file{1..3}.kt").is_match("file02.kt"));
        assert!(Glob::new("[abc].kt").is_match("b.kt"));
        assert!(Glob::new("[!abc].kt").is_match("d.kt"));
        assert!(Glob::new("**/test/**").is_match("x/test/y/Z.kt"));
        assert!(Glob::new("a\\#b").is_match("a#b"));
    }
}
