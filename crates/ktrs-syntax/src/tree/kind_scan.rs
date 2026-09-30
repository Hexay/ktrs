//! [`KindScan`]: the iterator behind [`Tree::find_kinds`](super::Tree::find_kinds).

use super::ElementId;

const CHUNK: usize = 64;
/// Raw kinds a scan can look for; unused slots hold [`NEVER`].
pub(super) const MAX_WANTED: usize = 12;
/// No raw kind: kinds are under 512, with or without the token bit.
pub(super) const NEVER: u16 = u16::MAX;

/// Matches 64 raw kinds at a time into a bitmask (the compares vectorize), then yields its set bits.
pub struct KindScan<'t> {
    kinds: &'t [u16],
    /// Element id of `kinds[0]`.
    base: usize,
    /// Start of the chunk `mask` describes.
    chunk: usize,
    mask: u64,
    wanted: [u16; MAX_WANTED],
}

impl<'t> KindScan<'t> {
    pub(super) fn new(kinds: &'t [u16], base: usize, wanted: [u16; MAX_WANTED]) -> Self {
        let mut scan = KindScan { kinds, base, chunk: 0, mask: 0, wanted };
        scan.mask = scan.chunk_mask(0);
        scan
    }

    fn chunk_mask(&self, start: usize) -> u64 {
        let wanted = self.wanted;
        let is_wanted = |k: u16| wanted.iter().fold(false, |a, &w| a | (w == k)) as u8;
        let mut hits = [0u8; CHUNK];
        match self.kinds.get(start..start + CHUNK) {
            Some(chunk) => {
                let chunk: &[u16; CHUNK] = chunk.try_into().unwrap();
                for (hit, &k) in hits.iter_mut().zip(chunk) {
                    *hit = is_wanted(k);
                }
            }
            None => {
                for (hit, &k) in hits.iter_mut().zip(self.kinds.get(start..).unwrap_or(&[])) {
                    *hit = is_wanted(k);
                }
            }
        }
        let mut mask = 0;
        for (g, bytes) in hits.chunks_exact(8).enumerate() {
            // Gathers the 0/1 bytes into the top byte: byte i lands in bit 56 + i.
            let packed = u64::from_le_bytes(bytes.try_into().unwrap()).wrapping_mul(0x0102_0408_1020_4080) >> 56;
            mask |= packed << (g * 8);
        }
        mask
    }
}

impl Iterator for KindScan<'_> {
    type Item = ElementId;

    fn next(&mut self) -> Option<ElementId> {
        while self.mask == 0 {
            self.chunk += CHUNK;
            if self.chunk >= self.kinds.len() {
                return None;
            }
            self.mask = self.chunk_mask(self.chunk);
        }
        let bit = self.mask.trailing_zeros() as usize;
        self.mask &= self.mask - 1;
        Some((self.base + self.chunk + bit) as ElementId)
    }
}
