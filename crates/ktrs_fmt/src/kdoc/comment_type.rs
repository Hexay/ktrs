//! Port of `CommentType.kt`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentType {
    Kdoc,
    Block,
    Line,
}

impl CommentType {
    /// The opening string of the comment.
    pub fn prefix(self) -> &'static str {
        match self {
            CommentType::Kdoc => "/**",
            CommentType::Block => "/*",
            CommentType::Line => "//",
        }
    }

    /// The closing string of the comment.
    pub fn suffix(self) -> &'static str {
        match self {
            CommentType::Kdoc | CommentType::Block => "*/",
            CommentType::Line => "",
        }
    }

    /// For multi line comments, the prefix at each comment line after the first one.
    pub fn line_prefix(self) -> &'static str {
        match self {
            CommentType::Kdoc => " * ",
            CommentType::Block => "",
            CommentType::Line => "// ",
        }
    }

    pub fn single_line_overhead(self) -> i32 {
        let suffix = self.suffix();
        (self.prefix().len() + suffix.len() + 1 + if suffix.is_empty() { 0 } else { 1 }) as i32
    }

    pub fn line_overhead(self) -> i32 {
        self.line_prefix().len() as i32
    }
}

pub fn is_kdoc_comment(s: &str) -> bool {
    s.starts_with("/**")
}

pub fn is_block_comment(s: &str) -> bool {
    s.starts_with("/*") && !s.starts_with("/**")
}

pub fn is_line_comment(s: &str) -> bool {
    s.starts_with("//")
}

pub fn comment_type(s: &str) -> CommentType {
    if is_kdoc_comment(s) {
        CommentType::Kdoc
    } else if is_block_comment(s) {
        CommentType::Block
    } else if is_line_comment(s) {
        CommentType::Line
    } else {
        panic!("Not a comment: {s}")
    }
}
