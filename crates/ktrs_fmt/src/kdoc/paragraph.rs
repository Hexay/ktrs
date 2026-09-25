//! Port of `Paragraph.kt`, part 1: state, cleanup and markup conversion (reflow is in
//! `paragraph_reflow.rs`). Paragraphs live in an arena; `prev`/`next` are arena indices.

use super::comment_type::CommentType;
use super::formatting_task::FormattingTask;
use super::kstring::{KChar, KStr, KString, w};
use super::options::KDocFormattingOptions;
use super::utilities::is_kdoc_tag;

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub(super) options: KDocFormattingOptions,
    pub(super) comment_type: CommentType,

    pub content: KString,

    pub prev: Option<usize>,
    pub next: Option<usize>,

    /// If true, this paragraph should be preceded by a blank line.
    pub separate: bool,
    /// If true, this paragraph continues the previous one (hanging indent including line 1).
    pub continuation: bool,
    /// Whether this paragraph may be empty (preformatted text can express repeated blank lines).
    pub allow_empty: bool,
    pub preformatted: bool,
    /// Is this a block paragraph? If so, it must start on its own line.
    pub block: bool,
    /// Is this paragraph specifying a kdoc tag like @param?
    pub doc: bool,
    /// The quote depth of this paragraph. 0 = not quoted, 1 = `>`, 2 = `> >`, etc.
    pub quoted: i32,
    pub table: bool,
    pub separator: bool,
    /// Should this paragraph use a hanging indent? Setting it implies [Self::block].
    hanging: bool,

    pub original_indent: i32,
    /// The indent to use for all lines in the paragraph.
    pub indent: KString,
    /// The indent for all lines if hanging, else for the second and subsequent lines.
    pub hanging_indent: KString,
}

impl Paragraph {
    pub fn new(task: &FormattingTask) -> Self {
        Paragraph {
            options: task.options,
            comment_type: task.comment_type,
            content: KString::new(),
            prev: None,
            next: None,
            separate: false,
            continuation: false,
            allow_empty: false,
            preformatted: false,
            block: false,
            doc: false,
            quoted: 0,
            table: false,
            separator: false,
            hanging: false,
            original_indent: 0,
            indent: KString::new(),
            hanging_indent: KString::new(),
        }
    }

    pub fn text(&self) -> &[u16] {
        &self.content
    }

    pub fn hanging(&self) -> bool {
        self.hanging
    }

    pub fn set_hanging(&mut self, value: bool) {
        self.block = true;
        self.hanging = value;
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn cleanup(&mut self) {
        if self.preformatted {
            return;
        }

        let mut s = self.content.clone();
        if self.options.convert_markup {
            s = self.convert_markup(&self.content);
        }
        if !self.options.allow_param_brackets {
            s = self.rewrite_params(&s);
        }
        self.content = s;
    }

    fn rewrite_params(&self, s: &[u16]) -> KString {
        let mut start = 0;
        let length = s.len();
        while start < length && s[start].is_whitespace() {
            start += 1;
        }
        if s.starts_with_at(w!("@param"), start) {
            start += "@param".len();
            while start < length && s[start].is_whitespace() {
                start += 1;
            }
            if start < length && {
                start += 1;
                s[start - 1] == '[' as u16
            } {
                while start < length && s[start].is_whitespace() {
                    start += 1;
                }
                let mut end = start;
                while end < length && s[end].is_java_identifier_part() {
                    end += 1;
                }
                if end > start {
                    let name = &s[start..end];
                    while end < length && s[end].is_whitespace() {
                        end += 1;
                    }
                    if end < length && {
                        end += 1;
                        s[end - 1] == ']' as u16
                    } {
                        while end < length && s[end].is_whitespace() {
                            end += 1;
                        }
                        return [w!("@param "), name, w!(" "), &s[end..]].concat();
                    }
                }
            }
        }

        s.to_vec()
    }

    fn convert_markup(&self, s: &[u16]) -> KString {
        // Whether the tag starts with a capital letter and needs to be cleaned, e.g. `@See` -> `@see`.
        let convert_kdoc_tag = is_kdoc_tag(s) && s[1].is_upper_case();

        if !convert_kdoc_tag
            && !s.iter().any(|&c| c == '<' as u16 || c == '&' as u16 || c == '{' as u16)
        {
            return s.to_vec();
        }

        let mut sb = KString::with_capacity(s.len());
        let mut i = 0;
        let n = s.len();

        if convert_kdoc_tag {
            sb.push('@' as u16);
            sb.push(s[1].lowercase_char());
            i += 2;
        }

        let mut code = false;
        let mut brackets = 0;
        while i < n {
            let c = s[i];
            i += 1;
            if c == '\\' as u16 {
                sb.push(c);
                if i + 1 < n {
                    sb.push(s[i]);
                    i += 1;
                }
                continue;
            } else if c == '`' as u16 {
                code = !code;
                sb.push(c);
                continue;
            } else if c == '[' as u16 {
                brackets += 1;
                sb.push(c);
                continue;
            } else if c == ']' as u16 {
                brackets -= 1;
                sb.push(c);
                continue;
            } else if code || brackets > 0 {
                sb.push(c);
                continue;
            } else if c == '<' as u16 {
                if s.starts_with_at(w!("b>"), i) || s.starts_with_at(w!("/b>"), i) {
                    // "<b>" or </b> -> "**"
                    sb.extend_from_slice(w!("**"));
                    if s[i] == '/' as u16 {
                        i += 1;
                    }
                    i += 2;
                    continue;
                }
                if s.starts_with_at(w!("i>"), i) || s.starts_with_at(w!("/i>"), i) {
                    // "<i>" or </i> -> "*"
                    sb.push('*' as u16);
                    if s[i] == '/' as u16 {
                        i += 1;
                    }
                    i += 2;
                    continue;
                }
                if s.starts_with_at(w!("em>"), i) || s.starts_with_at(w!("/em>"), i) {
                    // "<em>" or </em> -> "_"
                    sb.push('_' as u16);
                    if s[i] == '/' as u16 {
                        i += 1;
                    }
                    i += 3;
                    continue;
                }
                // <pre> is not converted here: it only appears in preformatted paragraphs.
            } else if c == '&' as u16 {
                if s.starts_with_ic_at(w!("lt;"), i) {
                    sb.push('<' as u16);
                    i += 3;
                    continue;
                }
                if s.starts_with_ic_at(w!("gt;"), i) {
                    sb.push('>' as u16);
                    i += 3;
                    continue;
                }
            } else if c == '{' as u16 {
                if s.starts_with_ic_at(w!("@param"), i) {
                    let curr = i + 6;
                    let mut end = s.index_of_char('}' as u16, curr);
                    if end == -1 {
                        end = n as i32;
                    }
                    let end = end as usize;
                    sb.push('[' as u16);
                    sb.extend_from_slice(s[curr..end].trim());
                    sb.push(']' as u16);
                    i = end + 1;
                    continue;
                } else if s.starts_with_ic_at(w!("@link"), i)
                    // kdoc does not render [symbol] as {@linkplain}, so converting would change output.
                    && !s.starts_with_ic_at(w!("@linkplain"), i)
                {
                    sb.push('[' as u16);
                    let mut curr = i + 5;
                    while curr < n {
                        let ch = s[curr];
                        curr += 1;
                        if ch.is_whitespace() {
                            break;
                        }
                        if ch == '}' as u16 {
                            curr -= 1;
                            break;
                        }
                    }
                    let mut skip = false;
                    while curr < n {
                        let ch = s[curr];
                        if ch == '}' as u16 {
                            sb.push(']' as u16);
                            curr += 1;
                            break;
                        } else if ch == '(' as u16 {
                            skip = true;
                        } else if !skip {
                            if ch == '#' as u16 {
                                if sb.last() != Some(&('[' as u16)) {
                                    sb.push('.' as u16);
                                }
                            } else {
                                sb.push(ch);
                            }
                        }
                        curr += 1;
                    }
                    i = curr;
                    continue;
                }
            }
            sb.push(c);
        }

        sb
    }
}
