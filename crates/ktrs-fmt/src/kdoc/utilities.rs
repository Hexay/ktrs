//! Port of `Utilities.kt`. All strings are UTF-16 (see `kstring`).

use super::kstring::{KChar, KStr, KString, w};
use super::options::KDocFormattingOptions;
use super::paragraph_list_builder_adjust::contains_only;

pub fn get_indent(width: i32) -> KString {
    vec![' ' as u16; width.max(0) as usize]
}

pub fn get_indent_size(indent: &[u16], options: &KDocFormattingOptions) -> i32 {
    let mut size = 0;
    for &c in indent {
        if c == '\t' as u16 {
            size += options.tab_width;
        } else {
            size += 1;
        }
    }
    size
}

/// Returns line number (1-based). Upstream defaults: `start_line = 1`, `start_offset = 0`.
pub fn get_line_number(source: &[u16], offset: usize, start_line: i32, start_offset: usize) -> i32 {
    let mut line = start_line;
    for &c in &source[start_offset..offset] {
        if c == '\n' as u16 {
            line += 1;
        }
    }
    line
}

/// `numberPattern = ^\d+([.)]) ` (Java `\d` is ASCII-only).
fn number_pattern_find(s: &[u16]) -> bool {
    let digits = s.iter().take_while(|&&c| (b'0' as u16..=b'9' as u16).contains(&c)).count();
    digits > 0
        && s.len() > digits + 1
        && (s[digits] == '.' as u16 || s[digits] == ')' as u16)
        && s[digits + 1] == ' ' as u16
}

pub fn is_list_item(s: &[u16]) -> bool {
    s.starts_with(w!("- "))
        || s.starts_with(w!("* "))
        || s.starts_with(w!("+ "))
        || (s.first().is_some_and(|c| c.is_digit()) && number_pattern_find(s))
        || s.starts_with_ic(w!("<li>"))
}

pub fn collapse_spaces(s: &[u16]) -> KString {
    if s.index_of(w!("  "), 0) == -1 {
        return s.trim_end().to_vec();
    }
    let mut sb = KString::with_capacity(s.len());
    let mut prev = s[0];
    for &c in s {
        if prev == ' ' as u16 && c == ' ' as u16 {
            continue;
        }
        sb.push(c);
        prev = c;
    }
    sb.trim_end().to_vec()
}

pub fn is_todo(s: &[u16]) -> bool {
    s.starts_with(w!("TODO:")) || s.starts_with(w!("TODO("))
}

pub fn is_header(s: &[u16]) -> bool {
    s.starts_with(w!("#")) || s.starts_with_ic(w!("<h"))
}

pub fn is_quoted(s: &[u16]) -> bool {
    s.starts_with(w!("> "))
}

pub fn is_directive_marker(s: &[u16]) -> bool {
    s.starts_with(w!("<!--")) || s.starts_with(w!("-->"))
}

/// Returns true if the string ends with a symbol that implies more text is coming, e.g. ":" or ","
pub fn is_expecting_more(s: &[u16]) -> bool {
    let Some(&last) = s.iter().rev().find(|c| !c.is_whitespace()) else { return false };
    last == ':' as u16 || last == ',' as u16
}

/// Does this String represent a divider line? Upstream default: `min_count = 3`.
pub fn is_line(s: &[u16], min_count: usize) -> bool {
    let line_of = |ch: u16| {
        s.first() == Some(&ch)
            && contains_only(s, &[ch, ' ' as u16])
            && s.iter().filter(|&&c| c == ch).count() >= min_count
    };
    line_of('-' as u16) || line_of('_' as u16)
}

pub fn is_kdoc_tag(s: &[u16]) -> bool {
    // Not using a hardcoded list here since tags can change over time
    if s.starts_with(w!("@")) && s.len() > 1 {
        for i in 1..s.len() {
            let c = s[i];
            if c.is_whitespace() {
                return i > 2;
            } else if !c.is_letter() || !c.is_lower_case() {
                if c == '[' as u16 && (s.starts_with(w!("@param")) || s.starts_with(w!("@property"))) {
                    // @param is allowed to use brackets: https://kotlinlang.org/docs/kotlin-doc.html#param-name
                    return true;
                } else if i == 1 && c.is_letter() && c.is_upper_case() {
                    // Allow capitalized tags such as @See (typos; convertMarkup fixes these).
                    return true;
                }
                return false;
            }
        }
        return true;
    }
    false
}

/// If this String represents a KDoc tag named [tag], returns the corresponding parameter name.
pub fn get_tag_name<'a>(s: &'a [u16], tag: &[u16]) -> Option<&'a [u16]> {
    let length = s.len();
    let mut start = 0;
    while start < length && s[start].is_whitespace() {
        start += 1;
    }
    if !s.starts_with_at(tag, start) {
        return None;
    }
    start += tag.len();

    while start < length && s[start].is_whitespace() {
        start += 1;
    }

    if start < length && s[start] == '[' as u16 {
        start += 1;
        while start < length && s[start].is_whitespace() {
            start += 1;
        }
    }

    let mut end = start;
    while end < length && s[end].is_java_identifier_part() {
        end += 1;
    }

    if end > start {
        return Some(&s[start..end]);
    }

    None
}

/// If this String represents a KDoc `@param` or `@property` tag, returns the parameter name.
pub fn get_param_name(s: &[u16]) -> Option<&[u16]> {
    get_tag_name(s, w!("@param")).or_else(|| get_tag_name(s, w!("@property")))
}

fn get_indent_at(start: i32, lookup: &dyn Fn(i32) -> u16) -> KString {
    let mut i = start - 1;
    while i >= 0 && lookup(i) != '\n' as u16 {
        i -= 1;
    }
    ((i + 1)..start).map(lookup).collect()
}

/// For a comment starting at offset [start] in a document of [max] characters (read through
/// [lookup]), compute the effective indent on the first line and on subsequent lines.
pub fn compute_indents(start: i32, lookup: &dyn Fn(i32) -> u16, max: i32) -> (KString, KString) {
    let original_indent = get_indent_at(start, lookup);
    let suffix = !original_indent.iter().all(|c| c.is_whitespace());
    let indent = if suffix {
        original_indent.iter().map(|&c| if c.is_whitespace() { c } else { ' ' as u16 }).collect()
    } else {
        original_indent.clone()
    };

    let secondary_indent = if suffix {
        // No good heuristic for the indent after a code line, so copy what the next lines do.
        let mut offset = start;
        while offset < max && lookup(offset) != '\n' as u16 {
            offset += 1;
        }
        offset += 1;
        let mut sb = KString::new();
        while offset < max {
            if lookup(offset) == '\n' as u16 {
                sb.clear();
            } else {
                let c = lookup(offset);
                if c.is_whitespace() {
                    sb.push(c);
                } else {
                    if c == '*' as u16 {
                        // The * of a comment is usually one space right of the / it aligns with.
                        sb.pop().expect("StringIndexOutOfBoundsException");
                    }
                    break;
                }
            }
            offset += 1;
        }
        sb
    } else {
        original_indent
    };

    (indent, secondary_indent)
}

/// Attempt to preserve the caret position across reformatting. Returns the delta in the new comment.
pub fn find_same_position(comment: &[u16], delta: i32, reformatted_comment: &[u16]) -> i32 {
    // Identical up to the delta: same new position
    for i in 0..comment.len().min(reformatted_comment.len()) {
        if i as i32 == delta {
            return delta;
        } else if comment[i] != reformatted_comment[i] {
            break;
        }
    }

    let mut i = comment.len() as i32 - 1;
    let mut j = reformatted_comment.len() as i32 - 1;
    if delta == i + 1 {
        return j + 1;
    }
    while i >= 0 && j >= 0 {
        if i == delta {
            return j;
        }
        if comment[i as usize] != reformatted_comment[j as usize] {
            break;
        }
        i -= 1;
        j -= 1;
    }

    let is_significant_char = |c: u16| c.is_whitespace() || c == '*' as u16;

    // Somewhere in the middle: search by character skipping insignificant ones (space, *, etc)
    let next_significant_char = |s: &[u16], from: i32| {
        let mut curr = from;
        while (curr as usize) < s.len() && is_significant_char(s[curr as usize]) {
            curr += 1;
        }
        curr
    };

    let mut offset = 0;
    let mut reformatted_offset = 0;
    while offset < delta && (reformatted_offset as usize) < reformatted_comment.len() {
        offset = next_significant_char(comment, offset);
        reformatted_offset = next_significant_char(reformatted_comment, reformatted_offset);
        if offset == delta {
            return reformatted_offset;
        }
        offset += 1;
        reformatted_offset += 1;
    }
    reformatted_offset
}

/// Upstream's `Iterable.maxOf` backport; panics (NoSuchElementException) on empty input.
pub fn max_of<T, R: PartialOrd>(items: &[T], selector: impl Fn(&T) -> R) -> R {
    let mut iter = items.iter();
    let mut max_value = selector(iter.next().expect("NoSuchElementException"));
    for item in iter {
        let v = selector(item);
        if max_value < v {
            max_value = v;
        }
    }
    max_value
}
