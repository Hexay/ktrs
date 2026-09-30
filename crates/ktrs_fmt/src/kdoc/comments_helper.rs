//! Port of `KDocCommentsHelper.kt` (implements gjf's `CommentsHelper`). The gjf `Input.Tok` it reads
//! is abstracted as [CommentTok] so the layout engine can plug in its own token type.

use super::formatter::KDocFormatter;
use super::kstring::{KStr, KString, ks, to_string, w};
use super::options::KDocFormattingOptions;

/// The parts of gjf's `Input.Tok` that `KDocCommentsHelper.rewrite` reads.
pub trait CommentTok {
    fn is_comment(&self) -> bool;
    fn original_text(&self) -> &str;
    fn is_javadoc_comment(&self) -> bool;
    fn is_slash_slash_comment(&self) -> bool;
}

/// Guava `CharMatcher.whitespace()` (Unicode White_Space) on a UTF-16 unit.
fn is_guava_whitespace(c: u16) -> bool {
    matches!(c, 0x09..=0x0D | 0x20 | 0x85 | 0xA0 | 0x1680 | 0x2000..=0x200A)
        || matches!(c, 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000)
}

/// gjf `Newlines.lineIterator`: lines including their terminator (`\r\n`, `\n` or `\r`).
fn line_iterator(input: &[u16]) -> Vec<&[u16]> {
    let mut lines = Vec::new();
    let mut curr = 0;
    let mut idx = 0;
    while idx < input.len() {
        let c = input[idx];
        if c == '\r' as u16 || c == '\n' as u16 {
            if c == '\r' as u16 && idx + 1 < input.len() && input[idx + 1] == '\n' as u16 {
                idx += 1;
            }
            idx += 1;
            lines.push(&input[curr..idx]);
            curr = idx;
        } else {
            idx += 1;
        }
    }
    if curr < input.len() {
        lines.push(&input[curr..]);
    }
    lines
}

fn spaces(n: i32) -> KString {
    vec![' ' as u16; n.max(0) as usize]
}

/// `^(//+)(?!noinspection|\$NON-NLS-\d+\$)[^\s/]`: returns the length of group 1 on a match.
fn line_comment_missing_space_prefix(line: &[u16]) -> Option<usize> {
    let slashes = line.iter().take_while(|&&c| c == '/' as u16).count();
    if slashes < 2 || slashes >= line.len() {
        return None;
    }
    let rest = &line[slashes..];
    let is_digit = |c: &u16| (b'0' as u16..=b'9' as u16).contains(c);
    let non_nls = rest.strip_prefix(w!("$NON-NLS-")).is_some_and(|after| {
        let digits = after.iter().take_while(|c| is_digit(c)).count();
        digits > 0 && after.get(digits) == Some(&('$' as u16))
    });
    let c = rest[0];
    let java_space = matches!(c, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D);
    if rest.starts_with(w!("noinspection")) || non_nls || java_space || c == '/' as u16 {
        return None;
    }
    Some(slashes)
}

/// `KDocCommentsHelper` rewrites KDoc comments for the layout engine.
pub struct KDocCommentsHelper {
    line_separator: KString,
    max_line_length: i32,
    kdoc_formatter: KDocFormatter,
}

impl KDocCommentsHelper {
    pub fn new(line_separator: &str, max_line_length: i32) -> Self {
        let mut options = KDocFormattingOptions::new(max_line_length, max_line_length);
        options.allow_param_brackets = true; // TODO Do we want this?
        options.convert_markup = false;
        options.set_nested_list_indent(4);
        options.optimal = false; // Use greedy line breaking for predictability.
        KDocCommentsHelper {
            line_separator: ks(line_separator),
            max_line_length,
            kdoc_formatter: KDocFormatter::new(options),
        }
    }

    /// gjf `CommentsHelper.rewrite(tok, maxWidth, column0)`; [_max_width] is unused upstream too.
    pub fn rewrite(&self, tok: &dyn CommentTok, _max_width: i32, column0: i32) -> String {
        if !tok.is_comment() {
            return tok.original_text().to_string();
        }
        if !tok.is_javadoc_comment() && self.is_unchanged_single_line(tok, column0) {
            return tok.original_text().to_string();
        }
        let mut text = tok.original_text().to_string();
        if tok.is_javadoc_comment() {
            text = self.kdoc_formatter.reformat_comment(&text, &" ".repeat(column0.max(0) as usize));
        }
        let text = ks(&text);
        let lines: Vec<KString> = line_iterator(&text)
            .into_iter()
            .map(|line| {
                let end = line.iter().rposition(|&c| !is_guava_whitespace(c)).map_or(0, |i| i + 1);
                line[..end].to_vec()
            })
            .collect();
        let result = if tok.is_slash_slash_comment() {
            self.indent_line_comments(&lines, column0)
        } else if self.javadoc_shaped(&lines) {
            self.indent_javadoc(&lines, column0)
        } else {
            self.preserve_indentation(&lines, column0)
        };
        to_string(&result)
    }

    /// Not upstream: whether [Self::rewrite] returns a non-javadoc comment as is because it is one line of
    /// printable ASCII with nothing to trim, no `//` space to add and nothing to wrap.
    fn is_unchanged_single_line(&self, tok: &dyn CommentTok, column0: i32) -> bool {
        let text = tok.original_text().as_bytes();
        let printable = |b: &u8| (0x20..=0x7E).contains(b);
        if text.first() != Some(&b'/') || text.last() == Some(&b' ') || !text.iter().all(printable) {
            return false;
        }
        if !tok.is_slash_slash_comment() {
            return true;
        }
        let slashes = text.iter().take_while(|&&b| b == b'/').count();
        (slashes == text.len() || text[slashes] == b' ') && text.len() as i32 + column0 <= self.max_line_length
    }

    /// For non-javadoc-shaped block comments, shift the entire block to the correct column, but
    /// do not adjust relative indentation.
    fn preserve_indentation(&self, lines: &[KString], column0: i32) -> KString {
        let mut builder = KString::new();

        // find the leftmost non-whitespace character in all trailing lines
        let mut start_col: i32 = -1;
        for line in &lines[1..] {
            let line_idx = line.iter().position(|&c| !is_guava_whitespace(c)).map_or(-1, |i| i as i32);
            if line_idx >= 0 && (start_col == -1 || line_idx < start_col) {
                start_col = line_idx;
            }
        }

        // output the first line at the current column
        builder.extend_from_slice(&lines[0]);

        // output all trailing lines with plausible indentation
        for line in &lines[1..] {
            builder.extend_from_slice(&self.line_separator);
            builder.extend_from_slice(&spaces(column0));
            // check that startCol is valid index, e.g. for blank lines
            if line.len() as i32 >= start_col {
                // A negative startCol throws StringIndexOutOfBoundsException upstream.
                builder.extend_from_slice(&line[usize::try_from(start_col).expect("StringIndexOutOfBounds")..]);
            } else {
                builder.extend_from_slice(line);
            }
        }
        builder
    }

    /// Wraps and re-indents line comments.
    fn indent_line_comments(&self, lines: &[KString], column0: i32) -> KString {
        let wrapped_lines = self.wrap_line_comments(lines, column0);
        let mut builder = KString::new();
        builder.extend_from_slice(wrapped_lines[0].trim());
        let indent_string = spaces(column0);
        for line in &wrapped_lines[1..] {
            builder.extend_from_slice(&self.line_separator);
            builder.extend_from_slice(&indent_string);
            builder.extend_from_slice(line.trim());
        }
        builder
    }

    fn wrap_line_comments(&self, lines: &[KString], column0: i32) -> Vec<KString> {
        let mut result = Vec::new();
        for original_line in lines {
            let mut line = original_line.clone();
            // Add missing leading spaces to line comments: `//foo` -> `// foo`.
            if let Some(length) = line_comment_missing_space_prefix(&line) {
                line = [&vec!['/' as u16; length][..], w!(" "), &line[length..]].concat();
            }
            if line.starts_with(w!("// MOE:")) {
                // don't wrap comments for https://github.com/google/MOE
                result.push(line);
                continue;
            }
            while line.len() as i32 + column0 > self.max_line_length {
                let mut idx = self.max_line_length - column0;
                // only break on whitespace characters, and ignore the leading `// `
                while idx >= 2 && !is_guava_whitespace(line[idx as usize]) {
                    idx -= 1;
                }
                if idx <= 2 {
                    break;
                }
                result.push(line[..idx as usize].to_vec());
                line = [w!("//"), &line[idx as usize..]].concat();
            }
            result.push(line);
        }
        result
    }

    /// Remove leading whitespace (trailing was already removed), and re-indent. Add a +1 indent
    /// before '*', and add the '*' if necessary.
    fn indent_javadoc(&self, lines: &[KString], column0: i32) -> KString {
        let mut builder = KString::new();
        builder.extend_from_slice(lines[0].trim());
        let indent = column0 + 1;
        let indent_string = spaces(indent);
        for line in &lines[1..] {
            builder.extend_from_slice(&self.line_separator);
            builder.extend_from_slice(&indent_string);
            let line = line.trim();
            if !line.starts_with(w!("*")) {
                builder.extend_from_slice(w!("* "));
            }
            builder.extend_from_slice(line);
        }
        builder
    }

    /// Returns true if the comment looks like javadoc
    fn javadoc_shaped(&self, lines: &[KString]) -> bool {
        let Some(first) = lines.first() else { return false };
        let first = first.trim();
        // if it's actually javadoc, we're done
        if first.starts_with(w!("/**")) {
            return true;
        }
        // if it's a block comment, check all trailing lines for '*'
        if !first.starts_with(w!("/*")) {
            return false;
        }
        lines[1..].iter().all(|line| line.trim().starts_with(w!("*")))
    }
}
