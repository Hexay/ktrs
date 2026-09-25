//! Port of `KDocFormattingOptions.kt`.

/// Options controlling how the [super::KDocFormatter] will behave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KDocFormattingOptions {
    /// Right hand side margin to write lines at.
    pub max_line_width: i32,
    /// Limit comment to be at most this many characters even if more would fit on the line.
    pub max_comment_width: i32,
    /// Whether to collapse multi-line comments that would fit on a single line into a single line.
    pub collapse_single_line: bool,
    /// Whether to collapse repeated spaces.
    pub collapse_spaces: bool,
    /// Whether to convert basic markup like `<b>bold</b>` into `**bold**`, `&lt;` into `<`, etc.
    pub convert_markup: bool,
    /// Whether to add punctuation where missing, such as ending sentences with a period.
    pub add_punctuation: bool,
    /// How many spaces to use for hanging indents in numbered lists and after block tags.
    pub hanging_indent: i32,
    /// When there are nested lists etc, how many spaces to indent by (see [Self::set_nested_list_indent]).
    nested_list_indent: i32,
    /// The tab width, for comments indented with tabs.
    pub tab_width: i32,
    /// Whether to perform optimal line breaking instead of greedy.
    pub optimal: bool,
    /// If true, reformat markdown tables such that the column markers line up.
    pub align_table_columns: bool,
    /// If true, moves any kdoc tags to the end of the comment and `@return` tags after `@param` tags.
    pub order_doc_tags: bool,
    /// If true, perform "alternative" formatting (IDE only; flips collapsing and optimal breaking).
    pub alternate: bool,
    /// Keep the alternate `@param[name]` bracket syntax instead of rewriting it.
    pub allow_param_brackets: bool,
}

impl Default for KDocFormattingOptions {
    fn default() -> Self {
        Self::new(72, 72)
    }
}

impl KDocFormattingOptions {
    /// Upstream's `maxCommentWidth` defaults to `maxLineWidth`; pass both explicitly.
    pub fn new(max_line_width: i32, max_comment_width: i32) -> Self {
        KDocFormattingOptions {
            max_line_width,
            max_comment_width,
            collapse_single_line: true,
            collapse_spaces: true,
            convert_markup: true,
            add_punctuation: false,
            hanging_indent: 2,
            nested_list_indent: 3,
            tab_width: 8,
            optimal: true,
            align_table_columns: true,
            order_doc_tags: true,
            alternate: false,
            allow_param_brackets: false,
        }
    }

    pub fn nested_list_indent(&self) -> i32 {
        self.nested_list_indent
    }

    pub fn set_nested_list_indent(&mut self, value: i32) {
        if value < 3 {
            panic!(
                "Nested list indent must be at least 3; if list items are only indented 2 spaces they \
                 will not be rendered as list items"
            );
        }
        self.nested_list_indent = value;
    }

    /// Creates a copy of this formatting object. Upstream does not copy `allowParamBrackets`.
    pub fn copy(&self) -> KDocFormattingOptions {
        let mut copy = KDocFormattingOptions::default();
        copy.max_line_width = self.max_line_width;
        copy.max_comment_width = self.max_comment_width;
        copy.collapse_single_line = self.collapse_single_line;
        copy.collapse_spaces = self.collapse_spaces;
        copy.hanging_indent = self.hanging_indent;
        copy.tab_width = self.tab_width;
        copy.align_table_columns = self.align_table_columns;
        copy.order_doc_tags = self.order_doc_tags;
        copy.add_punctuation = self.add_punctuation;
        copy.convert_markup = self.convert_markup;
        copy.nested_list_indent = self.nested_list_indent;
        copy.optimal = self.optimal;
        copy.alternate = self.alternate;
        copy
    }
}
