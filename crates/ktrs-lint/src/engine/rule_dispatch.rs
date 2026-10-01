//! Which rules a traversal calls `before_visit_child_nodes` on, per element type: built on first use of a type,
//! in rule order, so a node walks only its own rules instead of testing every rule's `visited_types`.

use ktrs_syntax::SyntaxKind;

use crate::rule::TokenSet;

const UNBUILT: (u32, u32) = (u32::MAX, 0);

pub(crate) struct RuleDispatch {
    /// Per element type: the `(start, len)` of its rule indices in `indices`.
    spans: Vec<(u32, u32)>,
    indices: Vec<u16>,
}

impl RuleDispatch {
    /// One flat buffer for every type's list, so a traversal allocates twice, not once per type.
    pub(crate) fn new(rule_count: usize) -> RuleDispatch {
        RuleDispatch {
            spans: vec![UNBUILT; SyntaxKind::DUMMY_HOLDER as usize + 1],
            indices: Vec::with_capacity(rule_count * 32),
        }
    }

    /// The indices of the rules whose `visited_types` (all of them, with `all`) include `kind`.
    pub(crate) fn rules_for(
        &mut self,
        kind: SyntaxKind,
        visited_types: impl Iterator<Item = Option<TokenSet>>,
        all: bool,
    ) -> (usize, usize) {
        let span = &mut self.spans[kind as usize];
        if *span == UNBUILT {
            let start = self.indices.len();
            self.indices.extend(
                visited_types
                    .enumerate()
                    .filter(|(_, types)| all || types.is_none_or(|t| t.contains(kind)))
                    .map(|(i, _)| i as u16),
            );
            *span = (start as u32, (self.indices.len() - start) as u32);
        }
        (span.0 as usize, span.1 as usize)
    }

    pub(crate) fn rule_at(&self, position: usize) -> usize {
        usize::from(self.indices[position])
    }
}
