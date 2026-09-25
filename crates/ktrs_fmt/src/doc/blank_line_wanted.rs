//! Port of `OpsBuilder.BlankLineWanted` (and its `SimpleBlankLine`/`ConditionalBlankLine`).

use super::output::BreakTag;

/// A request to add or remove a blank line in the output.
#[derive(Clone, Debug)]
pub enum BlankLineWanted {
    Simple(Option<bool>),
    Conditional(Vec<BreakTag>),
}

impl BlankLineWanted {
    /// Always emit a blank line.
    pub const YES: BlankLineWanted = BlankLineWanted::Simple(Some(true));
    /// Never emit a blank line.
    pub const NO: BlankLineWanted = BlankLineWanted::Simple(Some(false));
    /// Preserve blank lines from the input; overrides conditional blank lines.
    pub const PRESERVE: BlankLineWanted = BlankLineWanted::Simple(None);

    pub fn wanted(&self) -> Option<bool> {
        match self {
            BlankLineWanted::Simple(wanted) => *wanted,
            BlankLineWanted::Conditional(tags) => {
                if tags.iter().any(BreakTag::was_break_taken) {
                    Some(true)
                } else {
                    None
                }
            }
        }
    }

    pub fn merge(self, other: BlankLineWanted) -> BlankLineWanted {
        match (self, other) {
            (simple @ BlankLineWanted::Simple(_), _) => simple,
            (BlankLineWanted::Conditional(mut tags), BlankLineWanted::Conditional(other)) => {
                tags.extend(other);
                BlankLineWanted::Conditional(tags)
            }
            (BlankLineWanted::Conditional(_), other) => other,
        }
    }

    /// Emit a blank line if the given break is taken.
    pub fn conditional(break_tag: &BreakTag) -> BlankLineWanted {
        BlankLineWanted::Conditional(vec![break_tag.clone()])
    }
}
