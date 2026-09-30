//! The non-`checkFormatter` assertions of `KDocFormatterTest.kt` (`testWithOffset`,
//! `testWordBreaking`, `test236743270`) and its `loremize` helper.

use crate::kdoc::kstring::{KChar, KStr, KString, ks, to_string};
use crate::kdoc::utilities::find_same_position;

/// `source.indexOf("default")` + `findSamePosition` + the two assertions after it.
fn check_same_position(source: &str, reformatted: &str) {
    let source = ks(source);
    let reformatted = ks(reformatted);
    let initial_offset = source.index_of(&ks("default"), 0);
    let new_offset = find_same_position(&source, initial_offset, &reformatted);
    assert_ne!(new_offset, initial_offset);
    let start = new_offset as usize;
    assert_eq!(to_string(&reformatted[start..start + "default".len()]), "default");
}

#[test]
#[allow(non_snake_case)]
fn testWithOffset() {
    check_same_position(
        "/** Returns whether lint should check all warnings,\n * including those off by default */",
        "/**\n * Returns whether lint should check all warnings, including those\n * off by default\n */",
    );
}

#[test]
#[allow(non_snake_case)]
fn testWordBreaking() {
    check_same_position(
        "/** Returns whether lint should check all warnings,\n * including aaaaaa - off by default */",
        "/**\n * Returns whether lint should check all warnings, including\n * aaaaaa - off by default\n */",
    );
}

#[test]
fn test236743270() {
    let source = "/**\n * @return Amet do non adipiscing sed consequat duis non Officia ID (amet sed consequat non\n * adipiscing sed eiusmod), magna consequat.\n */";
    assert_eq!(loremize(source), source);
}

/// Test utility: derive an "equivalent" kdoc (same punctuation, whitespace, capitalization and
/// word lengths) with words from Lorem Ipsum.
fn loremize(s: &str) -> String {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt \
        ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco \
        laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in \
        voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat \
        non proident, sunt in culpa qui officia deserunt mollit anim id est laborum";
    let filtered: String = lorem.chars().filter(|c| c.is_alphabetic() || *c == ' ').collect();
    let lorem_words: Vec<KString> = filtered.to_lowercase().split(' ').map(ks).collect();
    let mut next = 0usize;

    let adjust_capitalization = |word: &[u16], original: &[u16]| -> KString {
        if original[0].is_upper_case() {
            if original.iter().all(|c| c.is_upper_case()) {
                ks(&to_string(word).to_uppercase())
            } else {
                let mut w = word.to_vec();
                w[0] = w[0].uppercase_char();
                w
            }
        } else {
            word.to_vec()
        }
    };

    let mut next_lorem = |word: &[u16]| -> KString {
        let length = word.len();
        let start = next;
        while next < lorem_words.len() {
            if lorem_words[next].len() == length {
                return adjust_capitalization(&lorem_words[next], word);
            }
            next += 1;
        }
        next = 0;
        while next < start {
            if lorem_words[next].len() == length {
                return adjust_capitalization(&lorem_words[next], word);
            }
            next += 1;
        }
        if length == 1 {
            return vec!['a' as u16 + (start % 26) as u16];
        }
        // No match for this word
        word.to_vec()
    };

    let s = ks(s);
    let mut sb = KString::new();
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if c.is_letter() {
            let mut end = i + 1;
            while end < s.len() && s[end].is_letter() {
                end += 1;
            }
            let word = &s[i..end];
            let keep = ["http", "https", "com"].iter().any(|k| word == ks(k).as_slice());
            if (i > 0 && s[i - 1] == '@' as u16) || keep {
                // Don't translate URL prefix/suffixes and doc tags
                sb.extend_from_slice(word);
            } else {
                sb.extend(next_lorem(word));
            }
            i = end;
        } else {
            sb.push(c);
            i += 1;
        }
    }
    to_string(&sb)
}
