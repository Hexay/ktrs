//! `detekt-core/.../Analyzer.kt`: runs the active rules over one file at a time.

use std::path::{Path, PathBuf};

use ktrs_psi::{KtElement, KtFile, PsiElement};

use super::path_filters::should_analyze_file;
use super::rule_descriptor::{AnalysisMode, RuleDescriptor};
use super::suppressions::is_suppressed_by;
use crate::api::{Entity, Finding, Issue, IssueEntity, IssueLocation, Location, Rule};
use crate::kt_file;

pub struct Analyzer {
    base_path: PathBuf,
    rules: Vec<RuleDescriptor>,
    #[allow(dead_code)] // read by the suppressors once they are ported (suppressions.rs)
    analysis_mode: AnalysisMode,
}

impl Analyzer {
    /// `rules`: the active ones (`Lifecycle` filters `ruleInstance.active`).
    pub fn new(base_path: PathBuf, rules: Vec<RuleDescriptor>, analysis_mode: AnalysisMode) -> Analyzer {
        Analyzer { base_path, rules, analysis_mode }
    }

    pub fn rules(&self) -> &[RuleDescriptor] {
        &self.rules
    }

    /// One file from its raw text: read as detekt reads it, parsed by its extension, analyzed.
    pub fn analyze_text(&self, path: &Path, raw_text: &str) -> Vec<Issue> {
        let text = kt_file::load_text(raw_text);
        let file = kt_file::parse_kt_file(&text, &path.to_string_lossy());
        self.analyze(&file, path)
    }

    /// `analyze(file, languageVersionSettings)`: findings in rule order, auto-correcting rules first. A panic is
    /// the exception that aborts the run upstream ("Analyzing <path> led to an exception.").
    pub fn analyze(&self, file: &KtFile, path: &Path) -> Vec<Issue> {
        let _context = kt_file::enter(file, path);
        let file_element = PsiElement::clone(file).upcast::<KtElement>();

        let mut correctable_rules: Vec<(&RuleDescriptor, Box<dyn Rule>)> = Vec::new();
        let mut other_rules: Vec<(&RuleDescriptor, Box<dyn Rule>)> = Vec::new();
        for rule_descriptor in &self.rules {
            let per_file = rule_descriptor.per_file();
            if !should_analyze_file(per_file.rule_set_filters.as_ref(), path, &self.base_path) {
                continue;
            }
            if !should_analyze_file(per_file.rule_filters.as_ref(), path, &self.base_path) {
                continue;
            }
            let rule_instance = &rule_descriptor.rule_instance;
            let rule = (rule_descriptor.rule_provider)(rule_descriptor.config.clone());
            if is_suppressed_by(&file_element, &rule_instance.id, &per_file.aliases, Some(&rule_instance.rule_set_id)) {
                continue;
            }
            if rule.auto_correct() { correctable_rules.push((rule_descriptor, rule)) } else { other_rules.push((rule_descriptor, rule)) }
        }

        let mut issues = Vec::new();
        for (rule_descriptor, mut rule) in correctable_rules.into_iter().chain(other_rules) {
            let rule_instance = &rule_descriptor.rule_instance;
            let aliases = &rule_descriptor.per_file().aliases;
            let mut findings = rule.visit_root_file(file);
            findings.retain(|finding| {
                !is_suppressed_by(&finding.entity.kt_element, &rule_instance.id, aliases, Some(&rule_instance.rule_set_id))
            });
            issues.extend(findings.into_iter().map(|finding| self.to_issue(finding, rule_descriptor)));
        }
        issues
    }

    fn to_issue(&self, finding: Finding, rule_descriptor: &RuleDescriptor) -> Issue {
        Issue {
            rule_instance: rule_descriptor.rule_instance.clone(),
            entity: self.entity_to_issue(finding.entity),
            references: finding.references.into_iter().map(|entity| self.entity_to_issue(entity)).collect(),
            message: finding.message,
            severity: rule_descriptor.rule_instance.severity,
            suppress_reasons: finding.suppress_reasons,
        }
    }

    fn entity_to_issue(&self, entity: Entity) -> IssueEntity {
        IssueEntity { signature: entity.signature, location: self.location_to_issue(entity.location) }
    }

    fn location_to_issue(&self, location: Location) -> IssueLocation {
        let path = location.path.strip_prefix(&self.base_path).map(Path::to_owned).unwrap_or(location.path);
        IssueLocation { source: location.source, end_source: location.end_source, text: location.text, path }
    }
}
