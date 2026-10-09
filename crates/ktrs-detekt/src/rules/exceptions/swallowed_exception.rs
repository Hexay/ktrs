//! `SwallowedException.kt`.

use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;

use ktrs_psi::{
    KtBlockExpression, KtCatchClause, KtDotQualifiedExpression, KtExpression, KtNameReferenceExpression, KtProperty, KtThrowExpression,
    PsiElement, kt_visitor_void,
};

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kotlin::{Regex, contains_ignore_case, remove_prefix, remove_suffix};
use crate::psi::is_allowed_exception_name;

/// `EXCEPTIONS_IGNORED_BY_DEFAULT`.
pub const EXCEPTIONS_IGNORED_BY_DEFAULT: [&str; 4] =
    ["InterruptedException", "MalformedURLException", "NumberFormatException", "ParseException"];

/// Exceptions should not be swallowed. This rule reports all instances where exceptions are `caught` and not
/// correctly passed (e.g. as a cause) into a newly thrown exception.
pub struct SwallowedException {
    base: RuleBase,
    ignored_exception_types: OnceCell<Vec<String>>,
    allowed_exception_name_regex: OnceCell<Regex>,
}

impl SwallowedException {
    pub fn new(config: Arc<dyn Config>) -> Self {
        SwallowedException {
            base: RuleBase::new(config, "The caught exception is swallowed. The original exception could be lost."),
            ignored_exception_types: OnceCell::new(),
            allowed_exception_name_regex: OnceCell::new(),
        }
    }

    fn ignored_exception_types(&self) -> &[String] {
        self.ignored_exception_types.get_or_init(|| {
            let exceptions = config_property::list(self.base.config.as_ref(), "ignoredExceptionTypes", &EXCEPTIONS_IGNORED_BY_DEFAULT);
            exceptions.iter().map(|it| remove_suffix(remove_prefix(it, "*"), "*").to_owned()).collect()
        })
    }

    fn allowed_exception_name_regex(&self) -> &Regex {
        self.allowed_exception_name_regex.get_or_init(|| {
            Regex::new(&config_property::string(self.base.config.as_ref(), "allowedExceptionNameRegex", "_|(ignore|expected).*"))
        })
    }

    fn is_exception_swallowed_or_unused(&self, catch_clause: &KtCatchClause) -> bool {
        self.is_exception_unused(catch_clause) || is_exception_swallowed(catch_clause)
    }

    fn is_exception_unused(&self, catch_clause: &KtCatchClause) -> bool {
        let parameter_name = catch_clause.catch_parameter().and_then(|parameter| parameter.name());
        let Some(catch_body) = catch_clause.catch_body() else { return true };
        !catch_body.any_descendant_of_type::<KtNameReferenceExpression>(|it| {
            let text = it.text_slice();
            self.ignored_exception_types().iter().any(|ignored| ignored == text) || parameter_name.as_deref() == Some(text)
        })
    }
}

impl Rule for SwallowedException {
    crate::rule_base!(SwallowedException);
}

crate::detekt_visitor! {
    impl SwallowedException {
        fn visit_catch_section(&mut self, catch_clause: &KtCatchClause) {
            if let Some(catch_parameter) = catch_clause.catch_parameter() {
                let exception_type = catch_parameter.type_reference().map(|type_reference| type_reference.text());
                if !self.ignored_exception_types().iter().any(|it| exception_type.as_ref().is_some_and(|t| contains_ignore_case(t, it)))
                    && self.is_exception_swallowed_or_unused(catch_clause)
                    && !is_allowed_exception_name(catch_clause, self.allowed_exception_name_regex())
                {
                    let description = self.base.description;
                    self.report(Finding::new(Entity::from(&catch_parameter), description));
                }
            }
            kt_visitor_void::visit_catch_section(self, catch_clause);
        }
    }
}

fn is_exception_swallowed(catch_clause: &KtCatchClause) -> bool {
    let parameter_name = catch_clause.catch_parameter().and_then(|parameter| parameter.name());
    let Some(catch_body) = catch_clause.catch_body() else { return false };
    catch_body.any_descendant_of_type::<KtThrowExpression>(|throw_expr| {
        let refs = parameter_references(throw_expr, parameter_name.as_deref(), &catch_body);
        !refs.is_empty() && refs.iter().all(|it| it.is::<KtDotQualifiedExpression>() && !it.parent().is_some_and(|p| p.is::<KtThrowExpression>()))
    })
}

fn parameter_references(throw_expr: &KtThrowExpression, parameter_name: Option<&str>, catch_body: &KtExpression) -> Vec<KtExpression> {
    let mut parameter_references_in_variables: HashMap<String, KtExpression> = HashMap::new();
    let Some(thrown_expression) = throw_expr.thrown_expression() else { return Vec::new() };
    thrown_expression
        .collect_descendants_of_type::<KtNameReferenceExpression>()
        .into_iter()
        .filter_map(|reference| {
            let reference_text = reference.text();
            if Some(reference_text.as_str()) == parameter_name {
                return Some(reference.upcast::<KtExpression>().get_qualified_expression_for_receiver_or_this());
            }
            if let Some(known) = parameter_references_in_variables.get(&reference_text) {
                return Some(known.clone());
            }
            let found = find_reference_in_variable(&reference, parameter_name, &reference_text, catch_body)?;
            parameter_references_in_variables.insert(reference_text, found.clone());
            Some(found)
        })
        .collect()
}

fn find_reference_in_variable(
    expression: &PsiElement,
    reference_name: Option<&str>,
    variable_name: &str,
    catch_body: &KtExpression,
) -> Option<KtExpression> {
    let block = expression.get_parent_of_type::<KtBlockExpression>(true)?;
    fn find(block: KtBlockExpression, reference_name: Option<&str>, variable_name: &str, catch_body: &KtExpression) -> Option<KtExpression> {
        let reference = block.find_descendant_of_type::<KtProperty>(|it| it.name().as_deref() == Some(variable_name)).and_then(|property| {
            let initializer = property.initializer()?;
            let compared = match initializer.cast::<KtDotQualifiedExpression>() {
                Some(dot_qualified) => dot_qualified.receiver_expression()?.text(),
                None => initializer.text(),
            };
            (Some(compared.as_str()) == reference_name).then_some(initializer)
        });
        if reference.is_some() {
            reference
        } else if PsiElement::eq(&block, catch_body) {
            None
        } else {
            find(block.get_parent_of_type::<KtBlockExpression>(true)?, reference_name, variable_name, catch_body)
        }
    }
    find(block, reference_name, variable_name, catch_body)
}
