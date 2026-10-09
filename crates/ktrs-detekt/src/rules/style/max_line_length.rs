//! `MaxLineLength.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtFile, KtStringTemplateExpression, PsiElement, kt_visitor_void};

use super::junk::find_kt_element_in_parents;
use crate::api::{Config, Entity, Finding, Location, Rule, RuleBase, config_property};
use crate::kt_file;
use crate::psi::{
    last_argument_matches_kotlin_reference_url_syntax, last_argument_matches_markdown_url_syntax, last_argument_matches_url,
};

const DEFAULT_IDEA_LINE_LENGTH: i32 = 120;

/// This rule reports lines of code which exceed a defined maximum line length.
pub struct MaxLineLength {
    base: RuleBase,
    max_line_length: OnceCell<i32>,
    exclude_package_statements: OnceCell<bool>,
    exclude_import_statements: OnceCell<bool>,
    exclude_comment_statements: OnceCell<bool>,
    exclude_raw_strings: OnceCell<bool>,
}

impl MaxLineLength {
    pub fn new(config: Arc<dyn Config>) -> Self {
        MaxLineLength {
            base: RuleBase::new(config, "Line detected, which is longer than the defined maximum line length in the code style."),
            max_line_length: OnceCell::new(),
            exclude_package_statements: OnceCell::new(),
            exclude_import_statements: OnceCell::new(),
            exclude_comment_statements: OnceCell::new(),
            exclude_raw_strings: OnceCell::new(),
        }
    }

    fn max_line_length(&self) -> i32 {
        *self.max_line_length.get_or_init(|| config_property::int(self.base.config.as_ref(), "maxLineLength", DEFAULT_IDEA_LINE_LENGTH))
    }

    fn exclude_package_statements(&self) -> bool {
        *self.exclude_package_statements.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "excludePackageStatements", true))
    }

    fn exclude_import_statements(&self) -> bool {
        *self.exclude_import_statements.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "excludeImportStatements", true))
    }

    fn exclude_comment_statements(&self) -> bool {
        *self.exclude_comment_statements.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "excludeCommentStatements", false))
    }

    fn exclude_raw_strings(&self) -> bool {
        *self.exclude_raw_strings.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "excludeRawStrings", true))
    }

    /// `offset`: the line's start, in bytes.
    fn is_valid_line(&self, file: &KtFile, offset: usize, line: &str) -> bool {
        utf16_length(line) as i64 <= i64::from(self.max_line_length())
            || self.is_ignored_statement(file, offset, line)
            || last_argument_matches_url(line)
            || last_argument_matches_markdown_url_syntax(line)
            || last_argument_matches_kotlin_reference_url_syntax(line)
    }

    fn is_ignored_statement(&self, file: &KtFile, offset: usize, line: &str) -> bool {
        self.contains_ignored_package_statement(line)
            || self.contains_ignored_import_statement(line)
            || self.contains_ignored_comment_statement(line)
            || self.contains_ignored_raw_string(file, offset, line)
    }

    fn contains_ignored_raw_string(&self, file: &KtFile, offset: usize, line: &str) -> bool {
        if !self.exclude_raw_strings() {
            return false;
        }

        let mut elements = find_kt_element_in_parents(file, offset, line.len());
        elements.sort_by_key(|it| it.text_offset());
        elements.last().is_some_and(is_inside_raw_string)
    }

    fn contains_ignored_package_statement(&self, line: &str) -> bool {
        if !self.exclude_package_statements() {
            return false;
        }

        line.trim_start().starts_with("package ")
    }

    fn contains_ignored_import_statement(&self, line: &str) -> bool {
        if !self.exclude_import_statements() {
            return false;
        }

        line.trim_start().starts_with("import ")
    }

    fn contains_ignored_comment_statement(&self, line: &str) -> bool {
        if !self.exclude_comment_statements() {
            return false;
        }

        let trimmed = line.trim_start();
        trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
    }
}

impl Rule for MaxLineLength {
    crate::rule_base!(MaxLineLength);
}

crate::detekt_visitor! {
    impl MaxLineLength {
        fn visit_kt_file(&mut self, file: &KtFile) {
            kt_visitor_void::visit_kt_file(self, file);

            let context = kt_file::containing_file(file);
            let max_line_length = i64::from(self.max_line_length());
            for (index, line) in context.text().split('\n').enumerate() {
                // Not upstream: a line of at most `max` bytes is valid whatever its UTF-16 length.
                if line.len() as i64 <= max_line_length {
                    continue;
                }
                let offset = context.line_start_offset(index);
                if self.is_valid_line(file, offset, line) {
                    continue;
                }
                let kt_element = find_first_meaningful_kt_element_in_parents(file, offset, line).unwrap_or_else(|| PsiElement::clone(file));
                let location = Location::of_range(&context, offset, offset + line.len());
                let description = self.base.description;
                self.report(Finding::new(Entity::from_location(&kt_element, location), description));
            }
        }
    }
}

fn utf16_length(line: &str) -> usize {
    if line.is_ascii() { line.len() } else { line.encode_utf16().count() }
}

/// `BLANK_OR_QUOTES.matches(it.text)`: only Java whitespace and `"`.
fn is_blank_or_quotes(text: &str) -> bool {
    text.chars().all(|c| matches!(c, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | '"'))
}

fn find_first_meaningful_kt_element_in_parents(file: &KtFile, offset: usize, line: &str) -> Option<PsiElement> {
    find_kt_element_in_parents(file, offset, line.len()).into_iter().find(|it| !is_blank_or_quotes(it.text_slice()))
}

fn is_inside_raw_string(element: &PsiElement) -> bool {
    element.get_parent_of_type::<KtStringTemplateExpression>(false).is_some()
}
