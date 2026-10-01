//! Port of ktlint-ruleset-standard `FilenameRule.kt` (https://kotlinlang.org/docs/coding-conventions.html#source-file-names).
//! A file with a single top level class (any kind, or an interface; KTIJ-21897), possibly with related top level
//! declarations, is named after the class; other files have a PascalCase name. Relaxed upstream (see its KDoc):
//! private classes don't count, a class used only as a return type does not force the name, and a single top
//! level object/typealias names the file. Files without `.kt` extension and `package.kt` are ignored.

use std::sync::LazyLock;

use ktrs_ast::psi::KtFile;
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{self, CLASS, FUN, IDENTIFIER, MODIFIER_LIST, OBJECT_DECLARATION, PROPERTY, TYPEALIAS, TYPE_REFERENCE};

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::{remove_surrounding, replace_first_char_uppercase_char, substring_after_last, substring_before};
use crate::rules::internal::{KotlinRegex, reg_ex_ignoring_diacritics_and_strokes_on_letters};

const VISITED_TYPES: TokenSet = TokenSet::create(&[SyntaxKind::FILE]);

#[derive(Default)]
pub struct FilenameRule {
    traversal_state: TraversalState,
}

impl RuleV2 for FilenameRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:filename")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal_state)
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let ast: &Ast = ast;
        if !ast.is_root(node) {
            return;
        }
        assert!(ast.is_file_element(node), "IllegalStateException: node is not FileASTNode");

        let file_path = KtFile::cast(ast, node).map(|it| it.virtual_file_path(ast));
        let Some(file_path) = file_path.filter(|it| it.ends_with(".kt") && !it.ends_with("package.kt")) else {
            // ignore all non ".kt" files (including ".kts")
            self.traversal_state.stop_traversal_of_ast();
            return;
        };

        let file_path = file_path.replace('\\', "/");
        let file_name = substring_before(substring_after_last(&file_path, "/"), ".");

        let top_level_class_declarations = top_level_declarations(ast, node, Some(CLASS));
        if let [top_level_class_declaration] = top_level_class_declarations.as_slice() {
            if has_top_level_declaration_not_extending(ast, node, &top_level_class_declaration.identifier) {
                should_match_pascal_case(ast, file_name, emit);
            } else {
                // A file with only one (non-private) top level class, and possibly some extension functions of that
                // class, is named after the class.
                should_match_class_name(ast, file_name, &top_level_class_declaration.identifier, emit);
            }
        } else {
            let top_level_declarations = top_level_declarations(ast, node, None);
            match top_level_declarations.as_slice() {
                [top_level_declaration] if matches!(top_level_declaration.element_type, OBJECT_DECLARATION | TYPEALIAS) => {
                    let pascal_case_identifier = replace_first_char_uppercase_char(&top_level_declaration.identifier);
                    should_match_file_name(ast, file_name, &pascal_case_identifier, emit);
                }
                _ => should_match_pascal_case(ast, file_name, emit),
            }
        }
        self.traversal_state.stop_traversal_of_ast();
    }
}

fn top_level_declarations(ast: &Ast, node: NodeId, element_type: Option<SyntaxKind>) -> Vec<TopLevelDeclaration<'_>> {
    let mut declarations: Vec<TopLevelDeclaration> = Vec::new();
    for declaration in ast
        .children(node)
        .filter(|&it| element_type.is_none_or(|t| ast.element_type(it) == t))
        .filter(|&it| does_not_have_private_modifier(ast, it))
        .filter_map(|it| to_top_level_declaration(ast, it))
    {
        if !declarations.contains(&declaration) {
            declarations.push(declaration);
        }
    }
    declarations
}

fn does_not_have_private_modifier(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, MODIFIER_LIST).is_none_or(|list| !ast.children(list).any(|it| ast.text_matches(it, "private")))
}

fn has_top_level_declaration_not_extending(ast: &Ast, node: NodeId, class_name: &str) -> bool {
    ast.children(node)
        .filter(|&it| does_not_have_private_modifier(ast, it))
        .any(|it| is_not_class_related_top_level_declaration(ast, it) || is_function_not_extending(ast, it, class_name))
}

fn is_not_class_related_top_level_declaration(ast: &Ast, node: NodeId) -> bool {
    NON_CLASS_RELATED_TOP_LEVEL_DECLARATION_TYPES.contains(&ast.element_type(node))
}

fn is_function_not_extending(ast: &Ast, node: NodeId, class_name: &str) -> bool {
    ast.element_type(node) == FUN
        && ast.find_child_by_type(node, TYPE_REFERENCE).is_none_or(|it| !ast.text(it).contains(class_name))
}

fn should_match_class_name(ast: &Ast, this: &str, class_name: &str, emit: &mut Emit<'_>) {
    if this != class_name {
        emit(
            ast,
            0,
            &format!(
                "File '{this}.kt' contains a single class, and possibly related top level declarations for that class. The file \
                 should be named after the class, '{class_name}.kt'"
            ),
            false,
        );
    }
}

fn should_match_file_name(ast: &Ast, this: &str, filename: &str, emit: &mut Emit<'_>) {
    if this != filename {
        emit(ast, 0, &format!("File '{this}.kt' contains a single top level declaration and should be named '{filename}.kt'"), false);
    }
}

fn should_match_pascal_case(ast: &Ast, this: &str, emit: &mut Emit<'_>) {
    if !PASCAL_CASE_REGEX.matches(this) {
        emit(ast, 0, &format!("File name '{this}.kt' should conform PascalCase"), false);
    }
}

#[derive(PartialEq, Eq)]
struct TopLevelDeclaration<'a> {
    element_type: SyntaxKind,
    identifier: &'a str,
}

fn to_top_level_declaration(ast: &Ast, node: NodeId) -> Option<TopLevelDeclaration<'_>> {
    ast.find_child_by_type(node, IDENTIFIER).map(|it| TopLevelDeclaration {
        element_type: ast.element_type(node),
        identifier: remove_surrounding(ast.leaf_text(it), "`", "`"),
    })
}

static PASCAL_CASE_REGEX: LazyLock<KotlinRegex> =
    LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("^[A-Z][A-Za-z\\d]*$"));
const NON_CLASS_RELATED_TOP_LEVEL_DECLARATION_TYPES: [SyntaxKind; 3] = [OBJECT_DECLARATION, TYPEALIAS, PROPERTY];
