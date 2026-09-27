//! Port of `RedundantImportDetector.kt` (lines 31-193). The `takeX(directive) { super.visitX() }`
//! callbacks become `enter_x` / `leave_x` around the caller's walk of the directive's subtree.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

use ktrs_psi::{
    FqName, ImportPath, KDocImpl, KDocLink, KDocName, KDocSection, KDocTag, KtImportDirective, KtImportList,
    KtPackageDirective, KtReferenceExpression, PsiElement,
};

const OPERATORS: &[&str] = &[
    // Unary prefix operators
    "unaryPlus",
    "unaryMinus",
    "not",
    // Increments and decrements
    "inc",
    "dec",
    // Arithmetic operators
    "plus",
    "minus",
    "times",
    "div",
    "rem",
    "mod", // deprecated
    "rangeTo",
    // 'In' operator
    "contains",
    // Indexed access operator
    "get",
    "set",
    // Invoke operator
    "invoke",
    // Augmented assignments
    "plusAssign",
    "minusAssign",
    "timesAssign",
    "divAssign",
    "remAssign",
    "modAssign", // deprecated
    // Equality and inequality operators
    "equals",
    // Comparison operators
    "compareTo",
    // Iterator operators
    "iterator",
    "next",
    "hasNext",
    // Bitwise operators
    "and",
    "or",
    // Property delegation operators
    "getValue",
    "setValue",
    "provideDelegate",
    // assign operator - Gradle compiler plugin
    "assign",
];

/// `Regex("component\\d+").matches(s)`.
fn matches_component_operator(s: &str) -> bool {
    s.strip_prefix("component").is_some_and(|d| !d.is_empty() && d.chars().all(is_java_regex_digit))
}

/// Java regex `\d` without UNICODE_CHARACTER_CLASS.
fn is_java_regex_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// `Regex("^@(param|property) (.+)").matches(s)`: `.` excludes Java line terminators.
fn matches_kdoc_tag_skip_first_reference(s: &str) -> bool {
    let rest = s.strip_prefix("@param ").or_else(|| s.strip_prefix("@property "));
    rest.is_some_and(|r| !r.is_empty() && !r.contains(['\n', '\r', '\u{85}', '\u{2028}', '\u{2029}']))
}

pub struct RedundantImportDetector {
    pub enabled: bool,
    this_package: Option<FqName>,
    used_references: HashSet<String, BuildHasherDefault<FxHasher>>,
    /// `lateinit` upstream; always set because every parsed file has an import list.
    import_clean_up_candidates: Vec<ImportCandidate>,
    is_package_element: bool,
    is_import_element: bool,
}

impl RedundantImportDetector {
    pub fn new(enabled: bool) -> RedundantImportDetector {
        RedundantImportDetector {
            enabled,
            this_package: None,
            // Upstream seeds this with OPERATORS; it is only queried for candidates, which exclude them.
            used_references: HashSet::default(),
            import_clean_up_candidates: Vec::new(),
            is_package_element: false,
            is_import_element: false,
        }
    }

    /// `takePackageDirective(directive) { super.visitPackageDirective() }` as a pair: call this before
    /// visiting the directive's subtree and [`Self::leave_package_directive`] after it.
    pub fn enter_package_directive(&mut self, directive: &KtPackageDirective) {
        if !self.enabled {
            return;
        }
        self.this_package = Some(directive.fq_name());
        self.is_package_element = true;
    }

    pub fn leave_package_directive(&mut self) {
        self.is_package_element = false;
    }

    /// `takeImportList(importList) { super.visitImportList() }`; see [`Self::enter_package_directive`].
    pub fn enter_import_list(&mut self, import_list: &KtImportList) {
        if !self.enabled {
            return;
        }
        self.import_clean_up_candidates = import_list
            .imports()
            .into_iter()
            .filter_map(|import| {
                let import_path = import.import_path()?;
                let identifier = identifier(&import_path)?;
                (import.is_valid_import()
                    && !OPERATORS.contains(&identifier.as_str())
                    && !matches_component_operator(&identifier))
                .then_some(ImportCandidate { import, identifier, imported_fq_name: import_path.fq_name })
            })
            .collect();
        self.is_import_element = true;
    }

    pub fn leave_import_list(&mut self) {
        self.is_import_element = false;
    }

    pub fn take_kdoc(&mut self, kdoc: &KDocImpl) {
        for kdoc_section in kdoc.get_children_of_type::<KDocSection>() {
            let tag_links: Vec<KDocLink> = kdoc_section
                .get_children_of_type::<KDocTag>()
                .into_iter()
                .flat_map(|tag| {
                    let tag_links = tag.get_children_of_type::<KDocLink>();
                    if matches_kdoc_tag_skip_first_reference(&tag.text()) {
                        tag_links.into_iter().skip(1).collect()
                    } else {
                        tag_links
                    }
                })
                .collect();

            let links = kdoc_section.get_children_of_type::<KDocLink>().into_iter().chain(tag_links);

            for link in links {
                for name in link.get_children_of_type::<KDocName>() {
                    if let Some(first) = name.qualified_name().first() {
                        self.used_references.insert(first.trim_matches(['[', ']']).to_owned());
                    }
                }
            }
        }
    }

    pub fn take_reference_expression(&mut self, expression: &KtReferenceExpression) {
        if !self.enabled {
            return;
        }

        if !self.is_package_element && !self.is_import_element && !expression.has_children() {
            let name = expression.text_slice().trim_matches('`');
            if !self.used_references.contains(name) {
                self.used_references.insert(name.to_owned());
            }
        }
    }

    pub fn get_redundant_import_elements(&self) -> Vec<PsiElement> {
        if !self.enabled {
            return Vec::new();
        }

        let mut identifier_counts: HashMap<&str, usize> = HashMap::new();
        for candidate in &self.import_clean_up_candidates {
            *identifier_counts.entry(&candidate.identifier).or_default() += 1;
        }

        self.import_clean_up_candidates
            .iter()
            .filter(|candidate| {
                let is_used = self.used_references.contains(&candidate.identifier);
                let imported_fq_name = &candidate.imported_fq_name;
                // A backtick-escaped full path (import `foo.bar.baz`) is a single-segment FqName whose
                // parent is ROOT, which would wrongly match the default package.
                let is_bracket_escaped_path = imported_fq_name.short_name().is_some_and(|s| s.contains('.'));
                let is_from_this_package = !is_bracket_escaped_path && imported_fq_name.parent() == self.this_package;
                let has_alias = candidate.import.alias().is_some();
                let is_overload = identifier_counts[candidate.identifier.as_str()] > 1;
                // Remove if...
                !is_used || (is_from_this_package && !has_alias && !is_overload)
            })
            .map(|candidate| PsiElement::from(candidate.import.clone()))
            .collect()
    }
}

/// An import clean-up candidate with its [`identifier`] and `importedFqName`, each computed once.
struct ImportCandidate {
    import: KtImportDirective,
    identifier: String,
    imported_fq_name: FqName,
}

/// The imported short name, possibly an alias name, if any.
fn identifier(import_path: &ImportPath) -> Option<String> {
    let name = import_path.imported_name()?;
    let name = name.trim_matches('`');
    // A fully backtick-escaped path (import `foo.bar.baz`) is one name with dots; use its last segment.
    Some(match name.rfind('.') {
        Some(dot_index) => name[dot_index + 1..].to_owned(),
        None => name.to_owned(),
    })
}

/// rustc's FxHash: [`RedundantImportDetector::used_references`] hashes every reference name in the file.
#[derive(Default)]
struct FxHasher(u64);

impl FxHasher {
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
}

impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            self.add(u64::from_le_bytes(chunk.try_into().unwrap()));
        }
        let mut tail = [0u8; 8];
        tail[..chunks.remainder().len()].copy_from_slice(chunks.remainder());
        self.add(u64::from_le_bytes(tail));
    }

    fn write_u8(&mut self, byte: u8) {
        self.add(u64::from(byte));
    }

    fn finish(&self) -> u64 {
        // hashbrown picks buckets from the low bits, which the multiply leaves weakest.
        self.0.rotate_left(26)
    }
}
