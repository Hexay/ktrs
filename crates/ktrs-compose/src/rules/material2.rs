//! Port of `rules/Material2.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{FqName, KtCallExpression, KtDotQualifiedExpression, KtFile, KtReferenceExpression};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kotlin_utils::fq_name_plus;
use crate::core::util::kt_dot_qualified_expressions::dedup_using_outermost;
use crate::core::util::psi_elements::{find_all_children, range};

pub struct Material2;

impl ComposeKtVisitor for Material2 {
    fn is_opt_in(&self) -> bool {
        true
    }

    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let m2 = m2_fq_name();
        let mut allowed_fq_names: Vec<FqName> =
            config.get_set("allowedFromM2", &[]).iter().map(|it| fq_name_plus(&m2, it)).collect();
        allowed_fq_names.push(fq_name_plus(&m2, "icons"));
        let import_list = file.import_list(ast);
        let imports: Vec<_> = import_list
            .map(|it| it.imports(ast))
            .unwrap_or_default()
            .into_iter()
            .filter(|it| it.imported_fq_name(ast).is_some_and(|fq| starts_with(&fq, &m2)))
            .filter(|directive| {
                !allowed_fq_names.iter().any(|it| directive.imported_fq_name(ast).is_some_and(|fq| starts_with(&fq, it)))
            })
            .collect();
        for directive in imports {
            emitter.report(ast, directive.node(), DISALLOWED_USAGE_OF_MATERIAL2, false);
        }
        let mut dot_qualified = find_all_children::<KtDotQualifiedExpression>(ast, file.node());
        if let Some(imps) = import_list {
            let imps = range(ast, imps.node());
            dot_qualified.retain(|it| !imps.contains(&ast.start_offset(it.node())));
        }
        if let Some(pkg) = file.package_directive(ast) {
            let pkg = range(ast, pkg.node());
            dot_qualified.retain(|it| !pkg.contains(&ast.start_offset(it.node())));
        }
        let references: Vec<_> = dedup_using_outermost(ast, &dot_qualified)
            .into_iter()
            .filter(|it| has_reference_to_m2(ast, *it, &m2, &allowed_fq_names))
            .collect();
        for reference in references {
            emitter.report(ast, reference.node(), DISALLOWED_USAGE_OF_MATERIAL2, false);
        }
    }
}

fn has_reference_to_m2(ast: &Ast, expression: KtDotQualifiedExpression, m2: &FqName, allowlist: &[FqName]) -> bool {
    let Some(selector) = expression.selector_expression(ast) else { return false };
    let fqn = if let Some(call) = KtCallExpression::cast(ast, selector) {
        let receiver = expression.receiver_expression(ast).expect("NullPointerException: receiverExpression");
        let callee = call.callee_expression(ast).map_or_else(|| "null".to_owned(), |c| ast.text(c));
        FqName::new(&format!("{}.{callee}", ast.text(receiver)))
    } else if KtReferenceExpression::is(ast, selector) {
        FqName::new(&expression.text(ast))
    } else {
        return false;
    };
    starts_with(&fqn, m2) && !allowlist.iter().any(|it| starts_with(&fqn, it))
}

/// `FqName.startsWith(FqName)` (`FqNameUnsafe.startsWith`): equal, or `other` followed by a `.`; never for root.
fn starts_with(fq_name: &FqName, other: &FqName) -> bool {
    let (this, other) = (fq_name.as_string(), other.as_string());
    !this.is_empty()
        && this.len() >= other.len()
        && (this.len() == other.len() || this.as_bytes()[other.len()] == b'.')
        && this.as_bytes().starts_with(other.as_bytes())
}

/// `FqName.fromSegments(listOf("androidx", "compose", "material"))`.
fn m2_fq_name() -> FqName {
    FqName::new("androidx.compose.material")
}

pub const DISALLOWED_USAGE_OF_MATERIAL2: &str = "\
Compose Material 2 is disallowed by your configuration.
See https://mrmans0n.github.io/compose-rules/rules/#dont-use-material-2 for more information.";
