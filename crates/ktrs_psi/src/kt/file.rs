//! `KtFile`, `KtScript`, package and import directives, plus the `FqName`/`ImportPath` values they return.

use ktrs_syntax::Parse;
use ktrs_syntax::SyntaxKind::*;

use crate::element::PsiElement;
use crate::tokens::INSIDE_DIRECTIVE_EXPRESSIONS;
use crate::types::*;

impl KtFile {
    pub fn new(parse: &Parse) -> KtFile {
        PsiElement::new(parse.syntax().into()).upcast()
    }

    /// `getImportList()`: the first import list child.
    pub fn import_list(&self) -> Option<KtImportList> {
        self.find_child_by_class()
    }
}

impl PsiErrorElement {
    /// `getErrorDescription()`. The message lives in `parse` (one per error element, in tree preorder).
    pub fn error_description<'p>(&self, parse: &'p Parse) -> &'p str {
        let root = parse.syntax();
        let index = root.descendants().filter(|n| n.kind() == ERROR_ELEMENT).position(|n| Some(&n) == self.as_node());
        index.and_then(|i| parse.error_messages.get(i)).map_or("", String::as_str)
    }
}

impl KtScript {
    /// `getBlockExpression()`: upstream requires it (throws when absent; None here).
    pub fn block_expression(&self) -> Option<KtBlockExpression> {
        self.find_child_by_class()
    }
}

impl KtPackageDirective {
    pub fn package_name_expression(&self) -> Option<KtExpression> {
        self.get_stub_or_psi_children_set(INSIDE_DIRECTIVE_EXPRESSIONS).into_iter().next()
    }

    pub fn package_names(&self) -> Vec<KtSimpleNameExpression> {
        let mut package_names = Vec::new();
        let mut name_expression = self.package_name_expression();
        while let Some(qualified) = name_expression.as_ref().and_then(|e| e.cast::<KtQualifiedExpression>()) {
            if let Some(selector) = qualified.selector_expression().and_then(|s| s.cast::<KtSimpleNameExpression>()) {
                package_names.push(selector);
            }
            name_expression = qualified.receiver_expression();
        }
        if let Some(simple) = name_expression.and_then(|e| e.cast::<KtSimpleNameExpression>()) {
            package_names.push(simple);
        }
        package_names.reverse();
        package_names
    }

    pub fn fq_name(&self) -> FqName {
        let qualified_name = self.qualified_name();
        if qualified_name.is_empty() { FqName::root() } else { FqName::new(&qualified_name) }
    }

    pub fn qualified_name(&self) -> String {
        self.package_names().iter().map(|e| e.referenced_name()).collect::<Vec<_>>().join(".")
    }

    pub fn package_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type(PACKAGE_KEYWORD)
    }
}

impl KtImportList {
    pub fn imports(&self) -> Vec<KtImportDirective> {
        self.get_stub_or_psi_children(IMPORT_DIRECTIVE)
    }
}

impl KtImportDirective {
    pub fn imported_reference(&self) -> Option<KtExpression> {
        self.get_stub_or_psi_children_set(INSIDE_DIRECTIVE_EXPRESSIONS).into_iter().next()
    }

    pub fn alias(&self) -> Option<KtImportAlias> {
        self.get_stub_or_psi_child(IMPORT_ALIAS)
    }

    pub fn alias_name(&self) -> Option<String> {
        self.alias()?.name()
    }

    pub fn is_all_under(&self) -> bool {
        self.node().find_child_by_type(MUL).is_some()
    }

    /// `getImportedFqName()`; upstream throws for references that aren't names (None here).
    pub fn imported_fq_name(&self) -> Option<FqName> {
        fq_name_from_expression(self.imported_reference())
    }

    pub fn import_path(&self) -> Option<ImportPath> {
        let fq_name = self.imported_fq_name()?;
        Some(ImportPath { fq_name, is_all_under: self.is_all_under(), alias: self.alias_name() })
    }

    pub fn is_valid_import(&self) -> bool {
        !self.has_error_elements()
    }
}

fn fq_name_from_expression(expression: Option<KtExpression>) -> Option<FqName> {
    let expression = expression?;
    if let Some(dot) = expression.cast::<KtDotQualifiedExpression>() {
        let parent_fqn = fq_name_from_expression(dot.receiver_expression());
        let Some(child) = name_from_expression(dot.selector_expression()) else { return parent_fqn };
        return Some(parent_fqn?.child(&child));
    }
    Some(FqName::top_level(&expression.cast::<KtSimpleNameExpression>()?.referenced_name()))
}

fn name_from_expression(expression: Option<KtExpression>) -> Option<String> {
    Some(expression?.cast::<KtSimpleNameExpression>()?.referenced_name())
}

impl KtImportAlias {
    /// `getName()`: the alias identifier's text (backticks kept).
    pub fn name(&self) -> Option<String> {
        Some(self.name_identifier()?.text())
    }
}

/// `org.jetbrains.kotlin.name.FqName`: equality is by string; `parent`/`shortName` are structural when the
/// name was built with `child`/`topLevel`, else split at the last dot outside backticks.
#[derive(Clone, Debug)]
pub struct FqName {
    fq_name: String,
    structure: Option<Box<(FqName, String)>>,
}

impl PartialEq for FqName {
    fn eq(&self, other: &FqName) -> bool {
        self.fq_name == other.fq_name
    }
}

impl Eq for FqName {}

impl FqName {
    pub fn root() -> FqName {
        FqName { fq_name: String::new(), structure: None }
    }

    pub fn new(fq_name: &str) -> FqName {
        FqName { fq_name: fq_name.to_owned(), structure: None }
    }

    pub fn top_level(short_name: &str) -> FqName {
        FqName { fq_name: short_name.to_owned(), structure: Some(Box::new((FqName::root(), short_name.to_owned()))) }
    }

    pub fn child(&self, name: &str) -> FqName {
        let fq_name = if self.is_root() { name.to_owned() } else { format!("{}.{name}", self.fq_name) };
        FqName { fq_name, structure: Some(Box::new((self.clone(), name.to_owned()))) }
    }

    pub fn as_string(&self) -> &str {
        &self.fq_name
    }

    pub fn is_root(&self) -> bool {
        self.fq_name.is_empty()
    }

    /// `parent()`; upstream throws on the root (None here).
    pub fn parent(&self) -> Option<FqName> {
        if let Some(structure) = &self.structure {
            return Some(structure.0.clone());
        }
        if self.is_root() {
            return None;
        }
        Some(match index_of_last_dot_with_backticks_support(&self.fq_name) {
            Some(dot) => FqName::new(&self.fq_name[..dot]),
            None => FqName::root(),
        })
    }

    /// `shortName().asString()`; upstream throws on the root (None here).
    pub fn short_name(&self) -> Option<String> {
        if let Some(structure) = &self.structure {
            return Some(structure.1.clone());
        }
        if self.is_root() {
            return None;
        }
        Some(match index_of_last_dot_with_backticks_support(&self.fq_name) {
            Some(dot) => self.fq_name[dot + 1..].to_owned(),
            None => self.fq_name.clone(),
        })
    }
}

fn index_of_last_dot_with_backticks_support(fq_name: &str) -> Option<usize> {
    let mut in_backticks = false;
    for (i, b) in fq_name.bytes().enumerate().rev() {
        match b {
            b'.' if !in_backticks => return Some(i),
            b'`' => in_backticks = !in_backticks,
            _ => {}
        }
    }
    None
}

/// `org.jetbrains.kotlin.resolve.ImportPath`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportPath {
    pub fq_name: FqName,
    pub is_all_under: bool,
    pub alias: Option<String>,
}

impl ImportPath {
    /// `getImportedName()`: the alias, else the short name; None for star imports.
    pub fn imported_name(&self) -> Option<String> {
        if self.is_all_under {
            return None;
        }
        self.alias.clone().or_else(|| self.fq_name.short_name())
    }
}
