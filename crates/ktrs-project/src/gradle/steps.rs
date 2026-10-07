//! Spotless steps (`ktfmt()`, `ktlint()`, ktrs's `KtrsStep` / `KtrsKtlintStep`) and ktlint extension options.

use super::interpret::{Interp, strip};
use crate::findings::{KtfmtPartial, KtlintPartial, style};
use crate::ir::{Expr, Seg, Stmt, render};

pub(crate) fn ktlint_option(it: &Interp, p: &mut KtlintPartial, name: &str, value: &Expr) -> bool {
    match name {
        "version" | "ktlintVersion" => {
            p.version = it.scope.str(value);
            p.version.is_some()
        }
        "android" => {
            p.android = it.scope.bool(value);
            p.android.is_some()
        }
        "enableExperimentalRules" | "experimentalRules" | "setUseExperimental" | "experimental" => {
            p.experimental = it.scope.bool(value);
            p.experimental.is_some()
        }
        "additionalEditorconfig" | "editorConfigOverride" | "userData" | "withEditorConfigOverride" => {
            let map = it.scope.map(value);
            map.iter().for_each(|(k, v)| p.set_override(k, v));
            !map.is_empty()
        }
        "customRuleSets" | "withCustomRuleSets" => {
            let rule_sets = strings_or_render(it, value);
            rule_sets.iter().for_each(|r| p.add_rule_set(r));
            !rule_sets.is_empty()
        }
        _ => false,
    }
}

fn strings_or_render(it: &Interp, e: &Expr) -> Vec<String> {
    let items: Vec<&Expr> = match e {
        Expr::List(items) => items.iter().collect(),
        Expr::Path(segs) if segs.len() == 1 && matches!(segs[0].name.as_str(), "listOf" | "setOf" | "files") => {
            segs[0].args().iter().map(|a| &a.value).collect()
        }
        _ => vec![e],
    };
    items.into_iter().map(|i| it.scope.str(i).unwrap_or_else(|| render(i))).collect()
}

/// A statement inside Spotless `kotlin {}` / `kotlinGradle {}`; `false` if not a recognised step.
pub(crate) fn spotless_step(it: &mut Interp, segs: &[Seg], gradle: bool) -> bool {
    let first = &segs[0];
    let section = if gradle { "kotlinGradle" } else { "kotlin" };
    match first.name.as_str() {
        "target" => {
            let targets: Vec<String> = first.args().iter().flat_map(|a| it.scope.strings(&a.value)).collect();
            section_mut(it, gradle).targets.extend(targets);
            true
        }
        "ktfmt" if first.args.is_some() => {
            let mut k = KtfmtPartial::default();
            for s in &segs[1..] {
                if s.args.is_some() && style(&s.name).is_some() {
                    k.style = style(&s.name);
                } else if let (true, Some(block)) = (s.name == "configure", &s.block) {
                    ktfmt_configure(it, block, &mut k);
                }
            }
            it.note(format!("spotless {section} {}", render(&Expr::Path(segs.iter().map(no_block).collect()))));
            merge_ktfmt(it, gradle, k);
            true
        }
        "ktlint" if first.args.is_some() => {
            let mut p = KtlintPartial { version: first.arg(0).and_then(|v| it.scope.str(v)), ..Default::default() };
            for s in &segs[1..] {
                if let Some(v) = s.arg(0) {
                    ktlint_option(it, &mut p, &s.name, v);
                }
            }
            it.note(format!("spotless {section} ktlint({})", p.version.as_deref().unwrap_or("")));
            merge_ktlint(it, gradle, p);
            true
        }
        "addStep" => {
            if let Some(Expr::Path(step)) = first.arg(0) {
                ktrs_step(it, step, gradle);
            }
            true
        }
        _ => false,
    }
}

fn no_block(s: &Seg) -> Seg {
    Seg { block: None, ..s.clone() }
}

fn section_mut<'s>(it: &'s mut Interp, gradle: bool) -> &'s mut super::findings::Section {
    if gradle { &mut it.out.spotless_kotlin_gradle } else { &mut it.out.spotless_kotlin }
}

fn merge_ktfmt(it: &mut Interp, gradle: bool, k: KtfmtPartial) {
    section_mut(it, gradle).ktfmt.get_or_insert_with(Default::default).merge(&k);
}

fn merge_ktlint(it: &mut Interp, gradle: bool, p: KtlintPartial) {
    section_mut(it, gradle).ktlint.get_or_insert_with(Default::default).merge(&p);
}

/// `ktfmt().configure { it.setMaxWidth(120); it.blockIndent = 4 }`.
fn ktfmt_configure(it: &Interp, block: &[Stmt], k: &mut KtfmtPartial) {
    for stmt in block {
        match stmt {
            Stmt::Expr(Expr::Path(segs)) => {
                if let [s] = strip(segs)
                    && let Some(v) = s.arg(0).and_then(|v| it.text(v))
                {
                    k.set(&s.name, &v);
                }
            }
            Stmt::Assign(Expr::Path(lhs), rhs) => {
                if let (Some(s), Some(v)) = (lhs.last(), it.text(rhs)) {
                    k.set(&s.name, &v);
                }
            }
            _ => {}
        }
    }
}

/// `addStep(KtrsStep.create(KtrsOptions.kotlinlang().withMaxWidth(120)))`, `KtrsKtlintStep.create(..)`.
fn ktrs_step(it: &mut Interp, step: &[Seg], gradle: bool) {
    let Some(c) = (1..step.len()).find(|&i| step[i].name == "create") else { return };
    let options: &[Seg] = match step[c].arg(0) {
        Some(Expr::Path(o)) => o,
        _ => &[],
    };
    match step[c - 1].name.as_str() {
        "KtrsStep" => {
            let mut k = KtfmtPartial::default();
            for s in options.iter().filter(|s| s.args.is_some()) {
                if let Some(st) = style(&s.name) {
                    k.style = Some(st);
                } else if let Some(v) = s.arg(0).and_then(|v| it.text(v)) {
                    if s.name == "of" {
                        k.style = style(&v);
                    } else {
                        k.set(&s.name, &v);
                    }
                }
            }
            it.note(format!("spotless {} KtrsStep", if gradle { "kotlinGradle" } else { "kotlin" }));
            merge_ktfmt(it, gradle, k);
        }
        "KtrsKtlintStep" => {
            let mut p = KtlintPartial::default();
            for s in options {
                match (s.name.as_str(), s.arg(0)) {
                    ("of", Some(v)) => p.version = it.scope.str(v),
                    (name, Some(v)) => {
                        ktlint_option(it, &mut p, name, v);
                    }
                    _ => {}
                }
            }
            it.note(format!("spotless {} KtrsKtlintStep", if gradle { "kotlinGradle" } else { "kotlin" }));
            merge_ktlint(it, gradle, p);
        }
        _ => {}
    }
}
