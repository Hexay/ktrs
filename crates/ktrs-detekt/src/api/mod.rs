//! `detekt-api`: what rules are written against.

pub mod config;
pub mod config_property;
mod entity;
mod issue;
mod location;
mod rule;
mod signatures;

pub use config::{Config, Value};
pub use entity::{Entity, Finding};
pub use issue::{Issue, IssueEntity, IssueLocation, RuleInstance, Severity};
pub use location::{Location, SourceLocation, TextLocation};
pub use rule::{Rule, RuleBase, RuleName, RuleProvider, RuleSet, RuleSetId, RuleSetProvider};
