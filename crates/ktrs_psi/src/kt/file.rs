//! `KtFile`, `KtScript`, package and import directives, plus the `FqName`/`ImportPath` values they return.

use std::cell::OnceCell;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

use ktrs_syntax::Parse;
use ktrs_syntax::SyntaxKind::*;

use crate::cast::PsiType;
use crate::element::PsiElement;
use crate::tokens::INSIDE_DIRECTIVE_EXPRESSIONS;
use crate::types::*;

/// `KtFile`. Unlike the other views it caches its text: `file.text` is O(1) upstream and ktfmt's
/// passes read it repeatedly, while rebuilding it walks the whole tree.
#[derive(Clone, Debug)]
pub struct KtFile(PsiElement, OnceCell<Rc<str>>);

impl PartialEq for KtFile {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for KtFile {}

impl Hash for KtFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PsiType for KtFile {
    fn can_cast(e: &PsiElement) -> bool {
        e.is_file()
    }

    fn cast_unchecked(e: PsiElement) -> Self {
        KtFile(e, OnceCell::new())
    }

    fn psi(&self) -> &PsiElement {
        &self.0
    }
}

impl Deref for KtFile {
    type Target = PsiElement;

    fn deref(&self) -> &PsiElement {
        &self.0
    }
}

impl From<KtFile> for PsiElement {
    fn from(value: KtFile) -> PsiElement {
        value.0
    }
}

impl KtFile {
    pub fn new(parse: &Parse) -> KtFile {
        PsiElement::root(parse.tree.clone()).upcast()
    }

    /// [`KtFile::new`] for a parse of `text`, which becomes the cached text.
    pub fn with_text(parse: &Parse, text: &str) -> KtFile {
        let file = KtFile::new(parse);
        debug_assert_eq!(file.text_length(), text.len(), "not the parsed text");
        if file.text_length() == text.len() {
            let _ = file.1.set(text.into());
        }
        file
    }

    /// `getText()`, computed once per `KtFile` value.
    pub fn text(&self) -> String {
        self.1.get_or_init(|| self.0.text().into()).to_string()
    }

    /// `getImportList()`: the first import list child.
    pub fn import_list(&self) -> Option<KtImportList> {
        self.find_child_by_class()
    }
}

impl PsiErrorElement {
    /// `getErrorDescription()`. The message lives in `parse` (one per error element, in tree preorder).
    pub fn error_description<'p>(&self, parse: &'p Parse) -> &'p str {
        let tree = self.tree();
        let index = (0..self.id()).filter(|&e| tree.kind(e) == ERROR_ELEMENT && !tree.is_token(e)).count();
        parse.error_messages.get(index).map_or("", String::as_str)
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
    /// Shared so `child` doesn't deep-copy the parent chain.
    structure: Option<Rc<(FqName, String)>>,
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
        FqName { fq_name: short_name.to_owned(), structure: Some(Rc::new((FqName::root(), short_name.to_owned()))) }
    }

    pub fn child(&self, name: &str) -> FqName {
        let fq_name = if self.is_root() { name.to_owned() } else { format!("{}.{name}", self.fq_name) };
        FqName { fq_name, structure: Some(Rc::new((self.clone(), name.to_owned()))) }
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
