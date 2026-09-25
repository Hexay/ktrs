//! Kotlin/JVM `String` and `Char` semantics over UTF-16 code units. Upstream measures widths with
//! `String.length` (UTF-16 units), so the formatter works on `[u16]` throughout; see `KStr`/`KChar`.

use super::char_tables;

/// A Kotlin `String` / `StringBuilder`.
pub type KString = Vec<u16>;

/// Zero-cost UTF-16 view of an ASCII string literal.
macro_rules! w {
    ($s:literal) => {{
        const N: usize = $s.len();
        const A: [u16; N] = $crate::kdoc::kstring::ascii::<N>($s);
        &A as &'static [u16]
    }};
}
pub(crate) use w;

pub const fn ascii<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut a = [0u16; N];
    let mut i = 0;
    while i < N {
        assert!(b[i] < 0x80);
        a[i] = b[i] as u16;
        i += 1;
    }
    a
}

pub fn ks(s: &str) -> KString {
    s.encode_utf16().collect()
}

pub fn to_string(s: &[u16]) -> String {
    String::from_utf16_lossy(s)
}

fn in_table(table: &[(u16, u16)], c: u16) -> bool {
    table
        .binary_search_by(|&(lo, hi)| {
            if hi < c {
                std::cmp::Ordering::Less
            } else if lo > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Kotlin `Char` predicates (JVM: `java.lang.Character` on a single UTF-16 unit).
pub trait KChar: Copy {
    /// Kotlin `Char.isWhitespace()`: `Character.isWhitespace || Character.isSpaceChar`.
    fn is_whitespace(self) -> bool;
    fn is_letter(self) -> bool;
    fn is_digit(self) -> bool;
    fn is_letter_or_digit(self) -> bool;
    fn is_upper_case(self) -> bool;
    fn is_lower_case(self) -> bool;
    fn is_java_identifier_part(self) -> bool;
    fn lowercase_char(self) -> u16;
    fn uppercase_char(self) -> u16;
    /// Kotlin `Char.equals(other, ignoreCase = true)`.
    fn eq_ignore_case(self, other: u16) -> bool;
}

fn simple_case_map(c: u16, upper: bool) -> u16 {
    let Some(ch) = char::from_u32(c as u32) else { return c };
    // Java's simple (1:1) mapping; Rust only exposes full mappings, which expand these.
    if !upper && c == 0x0130 {
        return 'i' as u16;
    }
    let mut it: Box<dyn Iterator<Item = char>> =
        if upper { Box::new(ch.to_uppercase()) } else { Box::new(ch.to_lowercase()) };
    match (it.next(), it.next()) {
        (Some(m), None) if (m as u32) <= 0xFFFF => m as u16,
        _ => c,
    }
}

impl KChar for u16 {
    fn is_whitespace(self) -> bool {
        matches!(self, 0x09..=0x0D | 0x1C..=0x20 | 0xA0 | 0x1680 | 0x2000..=0x200A)
            || matches!(self, 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000)
    }

    fn is_letter(self) -> bool {
        in_table(&char_tables::LETTER, self)
    }

    fn is_digit(self) -> bool {
        in_table(&char_tables::DIGIT, self)
    }

    fn is_letter_or_digit(self) -> bool {
        self.is_letter() || self.is_digit()
    }

    fn is_upper_case(self) -> bool {
        in_table(&char_tables::UPPER_CASE, self)
    }

    fn is_lower_case(self) -> bool {
        in_table(&char_tables::LOWER_CASE, self)
    }

    fn is_java_identifier_part(self) -> bool {
        in_table(&char_tables::JAVA_IDENTIFIER_PART, self)
    }

    fn lowercase_char(self) -> u16 {
        simple_case_map(self, false)
    }

    fn uppercase_char(self) -> u16 {
        simple_case_map(self, true)
    }

    fn eq_ignore_case(self, other: u16) -> bool {
        if self == other {
            return true;
        }
        let a = self.uppercase_char();
        let b = other.uppercase_char();
        a == b || a.lowercase_char() == b.lowercase_char()
    }
}

/// Kotlin `String` operations. Indices are UTF-16 offsets; `-1` means "not found" as upstream.
pub trait KStr {
    fn trim(&self) -> &[u16];
    fn trim_start(&self) -> &[u16];
    fn trim_end(&self) -> &[u16];
    fn is_blank(&self) -> bool;
    fn index_of(&self, p: &[u16], from: usize) -> i32;
    fn index_of_char(&self, c: u16, from: usize) -> i32;
    fn last_index_of(&self, p: &[u16]) -> i32;
    fn contains_seq(&self, p: &[u16]) -> bool;
    /// `startsWith(prefix, startIndex)`.
    fn starts_with_at(&self, p: &[u16], at: usize) -> bool;
    /// `regionMatches` / `startsWith(prefix, startIndex, ignoreCase = true)`.
    fn starts_with_ic_at(&self, p: &[u16], at: usize) -> bool;
    fn starts_with_ic(&self, p: &[u16]) -> bool;
    fn ends_with_ic(&self, p: &[u16]) -> bool;
    fn equals_ic(&self, p: &[u16]) -> bool;
    fn index_of_ic(&self, p: &[u16]) -> i32;
    fn replace_seq(&self, from: &[u16], to: &[u16], ignore_case: bool) -> KString;
    /// `split(delimiter)` with a single-unit delimiter; keeps empty parts.
    fn split_on(&self, delim: u16) -> Vec<KString>;
    fn remove_prefix(&self, p: &[u16]) -> &[u16];
    fn remove_suffix(&self, p: &[u16]) -> &[u16];
}

impl KStr for [u16] {
    fn trim(&self) -> &[u16] {
        self.trim_start().trim_end()
    }

    fn trim_start(&self) -> &[u16] {
        let start = self.iter().position(|c| !c.is_whitespace()).unwrap_or(self.len());
        &self[start..]
    }

    fn trim_end(&self) -> &[u16] {
        let end = self.iter().rposition(|c| !c.is_whitespace()).map_or(0, |i| i + 1);
        &self[..end]
    }

    fn is_blank(&self) -> bool {
        self.iter().all(|c| c.is_whitespace())
    }

    fn index_of(&self, p: &[u16], from: usize) -> i32 {
        if p.len() > self.len() {
            return -1;
        }
        (from..=self.len() - p.len()).find(|&i| self[i..].starts_with(p)).map_or(-1, |i| i as i32)
    }

    fn index_of_char(&self, c: u16, from: usize) -> i32 {
        (from..self.len()).find(|&i| self[i] == c).map_or(-1, |i| i as i32)
    }

    fn last_index_of(&self, p: &[u16]) -> i32 {
        if p.len() > self.len() {
            return -1;
        }
        (0..=self.len() - p.len()).rev().find(|&i| self[i..].starts_with(p)).map_or(-1, |i| i as i32)
    }

    fn contains_seq(&self, p: &[u16]) -> bool {
        self.index_of(p, 0) != -1
    }

    fn starts_with_at(&self, p: &[u16], at: usize) -> bool {
        at <= self.len() && self[at..].starts_with(p)
    }

    fn starts_with_ic_at(&self, p: &[u16], at: usize) -> bool {
        at <= self.len()
            && self.len() - at >= p.len()
            && self[at..at + p.len()].iter().zip(p).all(|(&a, &b)| a.eq_ignore_case(b))
    }

    fn starts_with_ic(&self, p: &[u16]) -> bool {
        self.starts_with_ic_at(p, 0)
    }

    fn ends_with_ic(&self, p: &[u16]) -> bool {
        self.len() >= p.len() && self.starts_with_ic_at(p, self.len() - p.len())
    }

    fn equals_ic(&self, p: &[u16]) -> bool {
        self.len() == p.len() && self.starts_with_ic(p)
    }

    fn index_of_ic(&self, p: &[u16]) -> i32 {
        if p.len() > self.len() {
            return -1;
        }
        (0..=self.len() - p.len()).find(|&i| self.starts_with_ic_at(p, i)).map_or(-1, |i| i as i32)
    }

    fn replace_seq(&self, from: &[u16], to: &[u16], ignore_case: bool) -> KString {
        let mut out = KString::with_capacity(self.len());
        let mut i = 0;
        while i < self.len() {
            let hit = if ignore_case {
                self.starts_with_ic_at(from, i)
            } else {
                self[i..].starts_with(from)
            };
            if hit && !from.is_empty() {
                out.extend_from_slice(to);
                i += from.len();
            } else {
                out.push(self[i]);
                i += 1;
            }
        }
        out
    }

    fn split_on(&self, delim: u16) -> Vec<KString> {
        self.split(|&c| c == delim).map(<[u16]>::to_vec).collect()
    }

    fn remove_prefix(&self, p: &[u16]) -> &[u16] {
        self.strip_prefix(p).unwrap_or(self)
    }

    fn remove_suffix(&self, p: &[u16]) -> &[u16] {
        self.strip_suffix(p).unwrap_or(self)
    }
}
