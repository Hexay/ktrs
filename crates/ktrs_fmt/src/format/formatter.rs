//! Port of `Formatter.kt` (lines 43-210): the presets and the pass pipeline.

use ktrs_psi::{KtFile, KtImportDirective, PsiComment, PsiElement, PsiWhiteSpace};

use crate::doc::{
    CommentsHelper, DocBuilder, Input, JavaOutput, OpsBuilder, Output, Range, State, Tok, newlines,
};
use crate::kdoc::{CommentTok, KDocCommentsHelper, index_of_comment_escape_sequences};

use super::FormatError;
use super::formatter_context::FormatterContext;
use super::formatting_options::{FormattingOptions, TrailingCommaManagementStrategy};
use super::input::whitespace_tombstones::{
    index_of_whitespace_tombstone, replace_tombstone_with_trailing_whitespace,
};
use super::input::{KotlinInput, ParseError};
use super::kotlin_input_ast_visitor::KotlinInputAstVisitor;
use super::kotlin_text::{compare_utf16, convert_line_separators, convert_line_separators_to};
use super::multiline_string_formatter::{MultilineStringFormatter, may_have_trimmed_strings};
use super::redundant_element_manager::{add_redundant_elements, drop_redundant_elements};

pub const META_FORMAT: FormattingOptions = FormattingOptions {
    trailing_comma_management_strategy: TrailingCommaManagementStrategy::OnlyAdd,
    ..FormattingOptions::new(2, 4)
};

pub const GOOGLE_FORMAT: FormattingOptions = FormattingOptions::new(2, 2);

/// A format that attempts to reflect https://kotlinlang.org/docs/coding-conventions.html.
pub const KOTLINLANG_FORMAT: FormattingOptions = FormattingOptions::new(4, 4);

/// `format(code)` with [META_FORMAT].
pub fn format_meta(code: &str) -> Result<String, FormatError> {
    format(&META_FORMAT, code)
}

/// `format(code, removeUnusedImports)` with [META_FORMAT].
pub fn format_remove_unused_imports(code: &str, remove_unused_imports: bool) -> Result<String, FormatError> {
    format(&FormattingOptions { remove_unused_imports, ..META_FORMAT }, code)
}

pub fn format(options: &FormattingOptions, code: &str) -> Result<String, FormatError> {
    let (shebang, kotlin_code) = if code.starts_with("#!") {
        match code.split_once('\n') {
            Some(parts) => parts,
            // `split(limit = 2)` yields one part; destructuring its second throws.
            None => return Err(FormatError::Runtime("Index 1 out of bounds for length 1".to_owned())),
        }
    } else {
        ("", code)
    };
    check_escape_sequences(kotlin_code)?;

    let context = FormatterContext::new(convert_line_separators(kotlin_code))
        .transform(sorted_and_distinct_imports)?
        .transform(|it| drop_redundant_elements(it, options))?
        .transform(|it| add_redundant_elements(it, options))?
        .transform(|it| pretty_print(it, options, "\n"))?
        .transform(|it| add_redundant_elements(it, options))?;
    // Skips the re-parse the last pass would otherwise need after add_redundant_elements changed the code.
    let code = if may_have_trimmed_strings(&context.code) {
        context.transform(|it| Ok(MultilineStringFormatter::new(options.continuation_indent).format(it)))?.code
    } else {
        context.code
    };
    let code = convert_line_separators_to(&code, newlines::guess_line_separator(kotlin_code));
    Ok(if shebang.is_empty() { code } else { format!("{shebang}\n{code}") })
}

/// prettyPrint reflows 'code' using google-java-format's engine.
fn pretty_print(file: &KtFile, options: &FormattingOptions, line_separator: &str) -> Result<String, FormatError> {
    let code = file.text();
    let kotlin_input = KotlinInput::new(&code, file.as_node().expect("KtFile is a node"))?;
    let comments_helper = KDocCommentsHelperAdapter(KDocCommentsHelper::new(line_separator, options.max_width));
    let mut java_output = JavaOutput::new(line_separator, &kotlin_input, Box::new(comments_helper));
    let ops = {
        let mut builder = OpsBuilder::new(&kotlin_input, &mut java_output);
        file.accept(&mut create_ast_visitor(options, &mut builder));
        builder.sync(kotlin_input.get_text().len() as i32);
        builder.drain();
        builder.build()?
    };
    if options.debugging_print_ops_after_formatting {
        for op in &ops {
            println!("{op:?}");
        }
    }
    let mut doc = DocBuilder::new().with_ops(ops).build();
    doc.compute_breaks(java_output.get_comments_helper(), options.max_width, State::new(0, 0));
    doc.write(&mut java_output);
    java_output.flush();

    let token_range_set =
        kotlin_input.character_ranges_to_token_ranges(&[Range::closed_open(0, code.len() as i32)])?;
    Ok(replace_tombstone_with_trailing_whitespace(&JavaOutput::apply_replacements(
        &code,
        &java_output.get_format_replacements(&token_range_set),
    )))
}

fn create_ast_visitor<'b, 'a>(
    options: &FormattingOptions,
    builder: &'b mut OpsBuilder<'a>,
) -> KotlinInputAstVisitor<'b, 'a> {
    KotlinInputAstVisitor::new(*options, builder)
}

fn check_escape_sequences(code: &str) -> Result<(), ParseError> {
    let mut index = index_of_whitespace_tombstone(code);
    if index == -1 {
        index = index_of_comment_escape_sequences(code);
    }
    if index != -1 {
        return Err(ParseError::at_offset(
            "ktfmt does not support code which contains one of {\\u0003, \\u0004, \\u0005} character; escape it",
            code,
            index as usize,
        ));
    }
    Ok(())
}

fn sorted_and_distinct_imports(file: &KtFile) -> Result<String, FormatError> {
    let code = file.text();

    let Some(import_list) = file.import_list() else { return Ok(code) };
    if import_list.imports().is_empty() {
        return Ok(code);
    }

    let mut comment_list: Vec<PsiElement> = Vec::new();
    // Comments are moved, in order, to the top of the import list; other non-imports are errors.
    let mut element = import_list.first_child();
    while let Some(e) = element {
        if e.is::<PsiComment>() {
            comment_list.push(e.clone());
        } else if !e.is::<KtImportDirective>() && !e.is::<PsiWhiteSpace>() {
            return Err(ParseError::at_offset(format!("Imports not contiguous: {}", e.text()), &code, e.start_offset()).into());
        }
        element = e.next_sibling();
    }
    fn canonical_text(import_directive: &KtImportDirective) -> String {
        let fq_name = import_directive.imported_fq_name().map_or("null".to_owned(), |f| f.as_string().to_owned());
        let alias = import_directive.alias().map_or("null".to_owned(), |a| a.text().replace('`', ""));
        let all_under = if import_directive.is_all_under() { "*" } else { "" };
        format!("{fq_name} {alias} {all_under}")
    }

    let mut sorted_imports: Vec<(String, KtImportDirective)> =
        import_list.imports().into_iter().map(|i| (canonical_text(&i), i)).collect();
    sorted_imports.sort_by(|a, b| compare_utf16(&a.0, &b.0));
    sorted_imports.dedup_by(|b, a| a.0 == b.0);
    let imports_with_comments =
        comment_list.into_iter().chain(sorted_imports.into_iter().map(|(_, i)| PsiElement::from(i)));

    let body = imports_with_comments.map(|i| i.text()).collect::<Vec<_>>().join("\n");
    // Kludge for idempotency: add a trailing newline only when an inline comment follows the last import.
    let needs_terminator = body.rfind('\n').is_some_and(|it| body[it + 1..].contains("//"));
    let mut result = code;
    let replacement = if needs_terminator { body + "\n" } else { body };
    result.replace_range(import_list.start_offset()..import_list.end_offset(), &replacement);
    Ok(result)
}

/// ktfmt's `KDocCommentsHelper` behind gjf's `CommentsHelper` interface.
struct KDocCommentsHelperAdapter(KDocCommentsHelper);

struct CommentTokAdapter<'t>(&'t dyn Tok);

impl CommentTok for CommentTokAdapter<'_> {
    fn is_comment(&self) -> bool {
        self.0.is_comment()
    }

    fn original_text(&self) -> &str {
        self.0.get_original_text()
    }

    fn is_javadoc_comment(&self) -> bool {
        self.0.is_javadoc_comment()
    }

    fn is_slash_slash_comment(&self) -> bool {
        self.0.is_slash_slash_comment()
    }
}

impl CommentsHelper for KDocCommentsHelperAdapter {
    fn rewrite(&self, tok: &dyn Tok, max_width: i32, column0: i32) -> String {
        self.0.rewrite(&CommentTokAdapter(tok), max_width, column0)
    }
}
