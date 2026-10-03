//! Port of `rules/ContentTrailingLambda.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFunction, containing_kt_file};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::lambdas::{composable_lambda_types, is_composable_lambda, lambda_types};

pub struct ContentTrailingLambda;

impl ComposeKtVisitor for ContentTrailingLambda {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let file = containing_kt_file(ast, function.node()).expect("containingKtFile").node();
        let lambda_types = lambda_types(ast, file, config);
        let composable_lambda_types = composable_lambda_types(ast, file, config);
        let parameters = function.value_parameters(ast);
        let matching: Vec<_> = parameters
            .iter()
            .filter(|it| it.name(ast).as_deref() == Some("content"))
            .filter(|parameter| {
                parameter
                    .type_reference(ast)
                    .is_some_and(|t| is_composable_lambda(ast, t, &lambda_types, &composable_lambda_types))
            })
            .collect();
        // `singleOrNull { }`: none when several match.
        let candidate = if matching.len() == 1 { Some(*matching[0]) } else { None };
        if let Some(candidate) = candidate
            && Some(&candidate) != parameters.last()
        {
            emitter.report(ast, candidate.node(), CONTENT_SHOULD_BE_TRAILING_LAMBDA, false);
        }
    }
}

pub const CONTENT_SHOULD_BE_TRAILING_LAMBDA: &str = "\
A @Composable `content` parameter should be moved to be the trailing lambda in a composable function.
See https://mrmans0n.github.io/compose-rules/rules/#slots-for-main-content-should-be-the-trailing-lambda for more information.";
