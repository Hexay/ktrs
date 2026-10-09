//! Port of `Formatter.kt` (lines 41-245): the presets and the pass pipeline.

use ktrs_psi::{KtFile, KtImportDirective, PsiComment, PsiElement, PsiWhiteSpace};

use crate::doc::{CommentsHelper, DocBuilder, Input, JavaOutput, OpsBuilder, Output, Range, RangeSet, State, Tok};
use crate::kdoc::{CommentTok, KDocCommentsHelper};

use super::FormatError;
use super::formatter_context::FormatterContext;
use super::formatting_options::{FormattingOptions, TrailingCommaManagementStrategy};
use super::input::whitespace_tombstones::replace_tombstone_with_trailing_whitespace;
use super::input::{KotlinInput, ParseError};
use super::kotlin_code::{FileType, KotlinCode};
use super::kotlin_input_ast_visitor::KotlinInputAstVisitor;
use super::kotlin_text::{compare_utf16, is_blank};
use super::multiline_string_formatter::{MultilineStringFormatter, may_have_trimmed_strings};
use super::redundant_element_manager::{add_redundant_elements, drop_redundant_elements};

pub const META_FORMAT: FormattingOptions = FormattingOptions {
    trailing_comma_management_strategy: TrailingCommaManagementStrategy::OnlyAdd,
    ..FormattingOptions::new(2, 4)
};

pub const GOOGLE_FORMAT: FormattingOptions = FormattingOptions::new(2, 2);

/// A format that attempts to reflect https://kotlinlang.org/docs/coding-conventions.html.
pub const KOTLINLANG_FORMAT: FormattingOptions = FormattingOptions::new(4, 4);

/// `format(code, fileType)` with [META_FORMAT].
pub fn format_meta(code: &str, file_type: FileType) -> Result<String, FormatError> {
    format(&META_FORMAT, &KotlinCode::from(code, file_type)?, None)
}

/// Formats the Kotlin code given in `code` and returns it as a string.
///
/// `character_ranges`: zero-indexed character ranges to format, using closed-open bounds. When
/// given, only pretty-print replacements are limited to those ranges. Whole-file cleanup passes,
/// such as import cleanup and multiline string formatting, still run afterward, mirroring
/// google-java-format's cleanup-after-selection behavior.
pub fn format(
    options: &FormattingOptions,
    code: &KotlinCode,
    character_ranges: Option<&RangeSet>,
) -> Result<String, FormatError> {
    let formatted_code = match character_ranges {
        None => {
            let context = FormatterContext::new(code.clone())
                .transform(|it| sorted_and_distinct_imports(it, false))?
                .transform(|it| drop_redundant_elements(it, options))?
                .transform(|it| add_redundant_elements(it, options))?;
            let context = pretty_print_and_manage_trailing_commas(context, options, "\n")?;
            format_multiline_strings(context, options)?.code
        }
        Some(character_ranges) => {
            let partially_formatted_code = if character_ranges.is_empty() {
                code.clone()
            } else {
                let character_ranges = character_ranges.as_canonical_ranges();
                FormatterContext::new(code.clone())
                    .transform(|it| pretty_print(it, options, "\n", Some(&character_ranges)))?
                    .code
            };
            let context = FormatterContext::new(partially_formatted_code)
                .transform(|it| drop_redundant_elements(it, options))?
                .transform(|it| sorted_and_distinct_imports(it, true))?
                .transform(|it| add_redundant_elements(it, options))?;
            format_multiline_strings(context, options)?.code
        }
    };

    Ok(formatted_code.to_string())
}

/// The `MultilineStringFormatter` pass, without the parse it needs when it would change nothing.
fn format_multiline_strings(context: FormatterContext, options: &FormattingOptions) -> Result<FormatterContext, FormatError> {
    if !may_have_trimmed_strings(&context.code.code) {
        return Ok(context);
    }
    context.transform(|it| Ok(MultilineStringFormatter::new(options.continuation_indent).format(it)))
}

/// Pretty-prints & reprints while [add_redundant_elements] keeps adding trailing commas, so a comma
/// inserted after layout can't leave a line over the limit.
fn pretty_print_and_manage_trailing_commas(
    mut context: FormatterContext,
    options: &FormattingOptions,
    line_separator: &str,
) -> Result<FormatterContext, FormatError> {
    loop {
        let pretty_code = context.transform(|it| pretty_print(it, options, line_separator, None))?;
        let (new_code, changed) = pretty_code.transform_changed(|it| add_redundant_elements(it, options))?;
        if !changed {
            return Ok(new_code);
        }
        context = new_code;
    }
}

/// prettyPrint reflows 'code' using google-java-format's engine. `character_ranges` defaults to the whole file.
fn pretty_print(
    file: &KtFile,
    options: &FormattingOptions,
    line_separator: &str,
    character_ranges: Option<&[Range]>,
) -> Result<String, FormatError> {
    let code = file.text();
    let kotlin_input = KotlinInput::new(&code, file)?;
    let comments_helper = KDocCommentsHelperAdapter(KDocCommentsHelper::new(line_separator, options.max_width));
    let mut java_output = JavaOutput::new(line_separator, &kotlin_input, Box::new(comments_helper));
    let ops = {
        let mut builder = OpsBuilder::new(&kotlin_input, &mut java_output);
        let mut visitor = create_ast_visitor(options, &mut builder);
        file.accept(&mut visitor);
        if let Some(exception) = visitor.take_exception() {
            return Err(exception);
        }
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

    let whole_file = [Range::closed_open(0, code.len() as i32)];
    let token_range_set = kotlin_input.character_ranges_to_token_ranges(character_ranges.unwrap_or(&whole_file))?;
    Ok(replace_tombstone_with_trailing_whitespace(&JavaOutput::apply_replacements(
        &code,
        &java_output.get_format_replacements(&token_range_set),
    )))
}

fn create_ast_visitor<'b, 'a, 'o>(
    options: &FormattingOptions,
    builder: &'b mut OpsBuilder<'a, 'o>,
) -> KotlinInputAstVisitor<'b, 'a, 'o> {
    KotlinInputAstVisitor::new(*options, builder)
}

fn sorted_and_distinct_imports(file: &KtFile, trim_leading_whitespace: bool) -> Result<String, FormatError> {
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
    let replace_start = if trim_leading_whitespace && is_blank(&code[..import_list.start_offset()]) {
        0
    } else {
        import_list.start_offset()
    };
    let mut result = code;
    let replacement = if needs_terminator { body + "\n" } else { body };
    result.replace_range(replace_start..import_list.end_offset(), &replacement);
    Ok(result)
}

/// ktfmt's `KDocCommentsHelper` behind gjf's `CommentsHelper` interface.
struct KDocCommentsHelperAdapter(KDocCommentsHelper);

struct CommentTokAdapter<'t>(&'t Tok<'t>);

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
    fn rewrite(&self, tok: &Tok<'_>, max_width: i32, column0: i32) -> String {
        self.0.rewrite(&CommentTokAdapter(tok), max_width, column0)
    }
}
