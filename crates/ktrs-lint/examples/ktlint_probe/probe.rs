//! One file through the engine, recording what tools/ktlint-oracle's `ProbeRule`/`FileRun` record
//! (src/Probe.kt): per pass whether the tree changed and whether it diverged from a fresh parse.

use std::panic::{self, AssertUnwindSafe};
use std::path::Path;

use ktrs_ast::Ast;
use ktrs_lint::{AutocorrectDecision, Code, KtLintParseException, KtLintRuleEngine, LintError};
use ktrs_parser::{FileKind, parse_file};

pub struct PassResult {
    pub pass: usize,
    pub changed: bool,
    pub diverged: bool,
    pub mutated: Option<String>,
    pub reparsed: Option<String>,
}

#[derive(Default)]
pub struct FileResult {
    pub rel: String,
    pub passes: Vec<PassResult>,
    pub format: Vec<String>,
    pub lint: Vec<String>,
    pub failure: Option<String>,
    pub formatted: Option<String>,
}

pub fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n")
}

fn row(e: &LintError) -> String {
    let auto = if e.can_be_auto_corrected { "auto" } else { "manual" };
    format!("{}\t{}\t{}\t{auto}\t{}", e.line, e.col, e.rule_id.value(), esc(&e.detail))
}

fn normalize(text: &str) -> String {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    text.strip_prefix('\u{FEFF}').map(str::to_owned).unwrap_or(text)
}

fn fresh_dump(psi_file_name: &str, text: &str) -> String {
    let ast = Ast::from_parse(&parse_file(text, FileKind::from_file_name(psi_file_name)));
    ast.psi_to_string(ast.root(), psi_file_name)
}

/// `FileRun`: the per-file state the probe fills in at the end of every traversal.
struct FileRun<'a> {
    psi_file_name: &'a str,
    original: String,
    keep_dumps: bool,
    pass: usize,
    autocorrects_in_pass: usize,
    passes: Vec<PassResult>,
    prev_text: String,
    prev_dump: Option<String>,
    prev_diverged: bool,
}

impl FileRun<'_> {
    fn end_of_pass(&mut self, ast: &Ast) {
        self.pass += 1;
        let text = ast.text(ast.root());
        if self.autocorrects_in_pass == 0 && text == self.prev_text {
            self.passes.push(PassResult { pass: self.pass, changed: false, diverged: self.prev_diverged, mutated: None, reparsed: None });
        } else {
            let mutated = ast.psi_to_string(ast.root(), self.psi_file_name);
            let baseline = self.prev_dump.clone().unwrap_or_else(|| fresh_dump(self.psi_file_name, &self.original));
            let changed = mutated != baseline;
            let reparsed = changed.then(|| fresh_dump(self.psi_file_name, &text));
            let diverged = if changed { Some(&mutated) != reparsed.as_ref() } else { self.prev_diverged };
            self.passes.push(PassResult {
                pass: self.pass,
                changed,
                diverged,
                mutated: (changed && self.keep_dumps || diverged).then(|| mutated.clone()),
                reparsed: reparsed.filter(|_| diverged),
            });
            self.prev_dump = Some(mutated);
            self.prev_diverged = diverged;
            self.prev_text = text;
        }
        self.autocorrects_in_pass = 0;
    }
}

pub fn process(engine: &KtLintRuleEngine, file: &Path, rel: &str, dumps: bool, lint: bool) -> FileResult {
    let mut result = FileResult { rel: rel.to_owned(), ..FileResult::default() };
    let content = match std::fs::read(file) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => {
            result.failure = Some(format!("crash\t{}", esc(&e.to_string())));
            return result;
        }
    };
    let path = file.to_string_lossy().into_owned();
    let code = Code::from_file(&path, content);
    let mut run = FileRun {
        psi_file_name: &path,
        original: normalize(&code.content),
        keep_dumps: dumps,
        pass: 0,
        autocorrects_in_pass: 0,
        passes: Vec::new(),
        prev_text: normalize(&code.content),
        prev_dump: None,
        prev_diverged: false,
    };
    let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
        let run = std::cell::RefCell::new(&mut run);
        let formatted = engine.format_observed(
            &code,
            &mut |e| {
                let mut run = run.borrow_mut();
                result.format.push(format!("{}\t{}", run.pass + 1, row(e)));
                if e.can_be_auto_corrected {
                    run.autocorrects_in_pass += 1;
                }
                AutocorrectDecision::AllowAutocorrect
            },
            &mut |ast| run.borrow_mut().end_of_pass(ast),
        )?;
        if formatted != code.content {
            result.formatted = Some(formatted);
        }
        if lint {
            engine.lint(&code, &mut |e| result.lint.push(row(e)))?;
        }
        Ok::<(), KtLintParseException>(())
    }));
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(e)) => result.failure = Some(format!("parse\t{}:{} {}:{} {}", e.line, e.col, e.line, e.col, esc(&e.message))),
        Err(payload) => {
            let message = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
            result.failure = Some(format!("crash\t{}", esc(&message.unwrap_or_default())));
        }
    }
    result.passes = run.passes;
    result
}
