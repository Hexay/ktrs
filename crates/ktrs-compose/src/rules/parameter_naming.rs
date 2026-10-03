//! Port of `rules/ParameterNaming.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFunction, containing_kt_file};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::lambdas::{is_lambda, lambda_types};

pub struct ParameterNaming;

impl ComposeKtVisitor for ParameterNaming {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let file = containing_kt_file(ast, function.node()).expect("containingKtFile");
        let lambda_types = lambda_types(ast, file.node(), config);
        let allowed = config.get_set("allowedLambdaParameterNames", &[]);
        let errors: Vec<_> = function
            .value_parameters(ast)
            .into_iter()
            .filter(|it| it.type_reference(ast).is_some_and(|t| is_lambda(ast, t, &lambda_types)))
            .filter_map(|it| Some((it, it.name(ast)?)))
            .filter(|(_, name)| name.starts_with("on"))
            .filter(|(_, name)| !allowed.contains(name))
            .filter(|(_, name)| !EXCEPTIONS_IN_COMPOSE.contains(&name.as_str()))
            .filter(|(_, name)| is_past_tense(name))
            .map(|(it, _)| it)
            .collect();
        for error in errors {
            emitter.report(ast, error.node(), LAMBDA_PARAMETERS_IN_PRESENT_TENSE, false);
        }
    }
}

fn is_past_tense(name: &str) -> bool {
    !VERBS_PRESENT_TENSE_ENDING_IN_ED.iter().any(|it| name.ends_with(it))
        && (name.ends_with("ed") || IRREGULAR_VERBS_IN_PAST_TENSE.iter().any(|it| name.ends_with(it)))
}

// "Left" is deliberately missing upstream (adjective/adverb use).
const IRREGULAR_VERBS_IN_PAST_TENSE: &[&str] = &[
    "Arose", "Arisen", "Ate", "Awoke", "Awoken", "Beaten", "Became", "Been", "Began", "Begun", "Bent", "Bit", "Bitten",
    "Bled", "Blew", "Blown", "Bore", "Borne", "Bought", "Bound", "Bred", "Broke", "Broken", "Brought", "Built", "Burnt",
    "Burst", "Came", "Caught", "Chose", "Chosen", "Clung", "Crept", "Dealt", "Did", "Done", "Drank", "Drawn", "Dreamt",
    "Drew", "Driven", "Drove", "Drunk", "Eaten", "Fallen", "Fed", "Felt", "Fled", "Flew", "Flown", "Forbade",
    "Forbidden", "Forgave", "Forgiven", "Forgot", "Forgotten", "Fought", "Found", "Froze", "Frozen", "Gave", "Given",
    "Gone", "Got", "Gotten", "Grew", "Grown", "Had", "Heard", "Held", "Hid", "Hidden", "Hung", "Kept", "Knew", "Known",
    "Laid", "Lain", "Lay", "Led", "Lent", "Lit", "Lost", "Made", "Meant", "Met", "Paid", "Ran", "Rang", "Ridden",
    "Risen", "Rode", "Rose", "Rung", "Said", "Sang", "Sank", "Sat", "Saw", "Seen", "Sent", "Shaken", "Shone", "Shook",
    "Shot", "Showed", "Shown", "Shrank", "Shrunk", "Slept", "Slid", "Sold", "Sought", "Spent", "Spoke", "Spoken",
    "Sprang", "Sprung", "Spun", "Stole", "Stolen", "Stood", "Struck", "Stuck", "Stung", "Sung", "Sunk", "Swam", "Swept",
    "Swore", "Sworn", "Swum", "Swung", "Taken", "Taught", "Thought", "Threw", "Thrown", "Told", "Took", "Tore", "Torn",
    "Understood", "Was", "Went", "Were", "Woke", "Woken", "Won", "Wore", "Worn", "Wound", "Written", "Wrote",
];

const VERBS_PRESENT_TENSE_ENDING_IN_ED: &[&str] = &[
    "Bed", "Bleed", "Embed", "Exceed", "Feed", "Heed", "Need", "Proceed", "Seed", "Shed", "Shred", "Sled", "Speed",
    "Succeed", "Wed", "Weed",
];

const EXCEPTIONS_IN_COMPOSE: &[&str] = &["onFocusChanged", "onPlaced", "onValueChangeFinished", "onDone"];

pub const LAMBDA_PARAMETERS_IN_PRESENT_TENSE: &str = "\
Lambda parameters in a composable function should be in present tense, not past tense.
Examples: `onClick` and not `onClicked`, `onTextChange` and not `onTextChanged`, etc.
See https://mrmans0n.github.io/compose-rules/rules/#naming-parameters-properly for more information.";
