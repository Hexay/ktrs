//! Direct-mapped green token interner, kept per thread across parses. Tokens repeat constantly
//! (punctuation, keywords, indentation, common identifiers), and sharing them saves most token
//! allocations. The fixed-size table bounds the memory held between parses and stays in cache;
//! a collision just evicts. Sharing is unobservable: green trees are values. Nodes are not
//! interned: measured on the corpus, hashing them costs more than the allocations it saves.

use std::cell::Cell;

use ktrs_syntax::SyntaxKind;
use rowan::GreenToken;

const SLOTS: usize = 1 << 10;

thread_local! {
    static SPARE: Cell<Option<Interner>> = const { Cell::new(None) };
}

pub(super) struct Interner {
    tokens: Box<[Option<GreenToken>]>,
}

impl Interner {
    /// This thread's interner, or a fresh one if it is already in use (nested sinks).
    pub(super) fn take() -> Interner {
        SPARE.with(Cell::take).unwrap_or_else(|| Interner { tokens: vec![None; SLOTS].into_boxed_slice() })
    }

    pub(super) fn give_back(self) {
        SPARE.with(|spare| spare.set(Some(self)));
    }

    pub(super) fn token(&mut self, kind: rowan::SyntaxKind, text: &str) -> GreenToken {
        let slot = &mut self.tokens[hash(kind, text.as_bytes()) & (SLOTS - 1)];
        match slot {
            Some(token) if token.kind() == kind && token.text() == text => token.clone(),
            _ => slot.insert(GreenToken::new(kind, text)).clone(),
        }
    }
}

/// FxHash-style multiply-rotate over 8-byte words.
fn hash(kind: rowan::SyntaxKind, bytes: &[u8]) -> usize {
    let mut h = kind.0 as u64;
    let mut mix = |word: u64| h = (h.rotate_left(5) ^ word).wrapping_mul(0x517c_c1b7_2722_0a95);
    let (words, rest) = bytes.as_chunks::<8>();
    for &word in words {
        mix(u64::from_le_bytes(word));
    }
    let mut tail = [0u8; 8];
    tail[..rest.len()].copy_from_slice(rest);
    mix(u64::from_le_bytes(tail) ^ (bytes.len() as u64) << 56);
    (h ^ h >> 29) as usize
}

pub(super) fn raw(kind: SyntaxKind) -> rowan::SyntaxKind {
    rowan::SyntaxKind(kind as u16)
}
