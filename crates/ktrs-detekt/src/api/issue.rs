//! `detekt-api/.../Issue.kt` and `Severity.kt`.

use std::path::PathBuf;

use super::location::{SourceLocation, TextLocation};
use super::rule::RuleSetId;

/// The severity for each [`Issue`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub const ENTRIES: [Severity; 3] = [Severity::Error, Severity::Warning, Severity::Info];

    pub fn name(self) -> &'static str {
        match self {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
        }
    }
}

/// Represents a problem detected by detekt on the source code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub rule_instance: RuleInstance,
    pub entity: IssueEntity,
    pub references: Vec<IssueEntity>,
    pub message: String,
    pub severity: Severity,
    pub suppress_reasons: Vec<String>,
}

impl Issue {
    pub fn location(&self) -> &IssueLocation {
        &self.entity.location
    }

    /// `Issue.suppressed`.
    pub fn suppressed(&self) -> bool {
        !self.suppress_reasons.is_empty()
    }
}

/// `Issue.Entity`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueEntity {
    pub signature: String,
    pub location: IssueLocation,
}

/// `Issue.Location`: the path is relative to the base path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueLocation {
    pub source: SourceLocation,
    pub end_source: SourceLocation,
    pub text: TextLocation,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleInstance {
    pub id: String,
    pub rule_set_id: RuleSetId,
    pub url: Option<String>,
    pub description: String,
    pub severity: Severity,
    pub active: bool,
}
