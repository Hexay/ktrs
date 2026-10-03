//! Structure-aware mutation: edits happen at Kotlin token boundaries (from `ktrs_lexer`), so a mutated seed stays
//! mostly lexable and often parseable, which byte-level mutation of real Kotlin rarely achieves.

use ktrs_lexer::tokens_of;

/// Spliced in at token boundaries: tokens and small constructs the formatters and lint rules special-case.
const FRAGMENTS: &[&str] = &[
    "\n", "\n\n", " ", "\t", "\n    ", ";", ",", ".", "?.", "!!", "::", ":", "=", "==", "->", "..", "..<", "?:",
    "+", "-", "*", "/", "&&", "||", "!", "(", ")", "{", "}", "[", "]", "<", ">", "@", "_", "it", "x", "`a b`",
    "fun", "val", "var", "class", "object", "interface", "if", "else", "when", "for", "while", "do", "try",
    "catch", "finally", "return", "throw", "break", "continue", "is", "!is", "in", "!in", "as", "as?", "this",
    "super", "null", "true", "package a.b\n", "import a.b.C\n", "import a.b.*\n", "import a.b as c\n",
    "typealias T = Int\n", "private ", "internal ", "override ", "open ", "data ", "sealed ", "inline ",
    "suspend ", "operator ", "infix ", "lateinit ", "const ", "companion ", "enum ", "value ", "annotation ",
    "context(a: A) ", "@Suppress(\"x\") ", "@file:JvmName(\"F\")\n", "@Ann ", "@[A B] ", "<T>", "<in T, out R>",
    "<*>", "?", "fun f() {}\n", "fun f(a: Int, b: String = \"\",) = a\n", "fun <T> T.f(): T where T : Any = this\n",
    "class A(val a: Int) : B(), C {\n}\n", "object O\n", "enum class E { A, B, ; fun f() = 1 }\n",
    "val x: Int get() = 1\n", "var y = 0\n    private set\n", "if (a) b else c", "when (x) { 1, 2 -> a\n else -> {} }",
    "when { a -> b }", "try { a() } catch (e: E) { } finally { }", "for ((a, b) in c) {}", "while (true) {}",
    "do { } while (x)", "{ a, b -> a + b }", "{ it }", "f(a, b = 1, *c)", "a.b.c().d { }", "a?.let { }",
    "x[1, 2]", "(a)", "A::class", "::f", "this@A", "return@f", "l@ ", "label@ for (i in 0..1) break@label",
    "\"s\"", "\"$x\"", "\"${x + 1}\"", "\"\"\"\n  a\n  \"\"\".trimIndent()", "\"\"\"${'$'}\"\"\"", "'c'", "'\\n'",
    "1", "0x1F", "1L", "1.0f", "1_000", "// c\n", "/* c */", "/** Doc. */\n", "/**\n * Doc [a].\n *\n * @param a b\n */\n",
    "/*\n * c\n */\n", "// ktlint-disable\n", "// ktlint-enable\n", "@Suppress(\"ktlint\")\n", "// @formatter:off\n",
    "// @formatter:on\n", "#!/usr/bin/env kotlin\n", "init { }\n", "constructor() : this(1)\n", "by lazy { 1 }",
    "fun interface F { fun f() }\n", "expect ", "actual ", "external ", "tailrec ", "vararg ", "crossinline ",
    "noinline ", "reified ", "dynamic", "Unit", "Nothing?", "(Int) -> Unit", "suspend () -> Unit", "A.() -> B",
    "Array<out T>", "List<in T>", "Map<K, V>?", "T & Any",
];

const MAX_RANGE: usize = 12;

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// Byte offsets of every token boundary, 0 and the length included.
pub fn boundaries(text: &str) -> Vec<usize> {
    let mut out = vec![0];
    let mut at = 0;
    for token in tokens_of(text) {
        at += token.len as usize;
        if out.last() != Some(&at) && text.is_char_boundary(at) {
            out.push(at);
        }
    }
    if out.last() != Some(&text.len()) {
        out.push(text.len());
    }
    out
}

/// A run of 1..=MAX_RANGE whole tokens, as a byte range.
fn token_range(rng: &mut Rng, b: &[usize]) -> (usize, usize) {
    if b.len() < 2 {
        return (0, 0);
    }
    let i = rng.below(b.len() - 1);
    let j = (i + 1 + rng.below(MAX_RANGE)).min(b.len() - 1);
    (b[i], b[j])
}

fn pick<'a>(rng: &mut Rng, b: &'a [usize]) -> usize {
    b[rng.below(b.len())]
}

fn splice(text: &str, at: (usize, usize), with: &str) -> String {
    [&text[..at.0], with, &text[at.1..]].concat()
}

/// One token-level edit of `text`, or `None` to let the caller fall back to byte-level mutation.
pub fn mutate(text: &str, seed: u64) -> Option<String> {
    let mut rng = Rng::new(seed);
    let b = boundaries(text);
    let fragment = FRAGMENTS[rng.below(FRAGMENTS.len())];
    Some(match rng.below(9) {
        0 | 1 => {
            let at = pick(&mut rng, &b);
            splice(text, (at, at), fragment)
        }
        2 => splice(text, token_range(&mut rng, &b), ""),
        3 => splice(text, token_range(&mut rng, &b), fragment),
        4 | 5 => {
            let (s, e) = token_range(&mut rng, &b);
            let at = pick(&mut rng, &b);
            splice(text, (at, at), &text[s..e])
        }
        6 => {
            let (s, e) = token_range(&mut rng, &b);
            let to = token_range(&mut rng, &b);
            splice(text, to, &text[s..e])
        }
        7 => {
            let (s, e) = token_range(&mut rng, &b);
            let after = b.iter().copied().filter(|&x| x > e).collect::<Vec<_>>();
            if after.is_empty() {
                return None;
            }
            let f = after[rng.below(after.len().min(MAX_RANGE))];
            [&text[..s], &text[e..f], &text[s..e], &text[f..]].concat()
        }
        _ => return None,
    })
}

/// libFuzzer's `LLVMFuzzerCustomMutator` contract: mutates `data[..size]` in place within `max_size` bytes.
pub fn mutate_in_place(
    data: &mut [u8],
    size: usize,
    max_size: usize,
    seed: u32,
    fallback: fn(&mut [u8], usize, usize) -> usize,
) -> usize {
    let mutated = std::str::from_utf8(&data[..size]).ok().and_then(|text| mutate(text, seed.into()));
    match mutated {
        Some(out) if out.len() <= max_size => {
            data[..out.len()].copy_from_slice(out.as_bytes());
            out.len()
        }
        _ => fallback(data, size, max_size),
    }
}

/// Token-level crossover: a prefix of one input joined to a suffix of the other, or a run of `b`'s tokens
/// spliced into `a`.
pub fn crossover(a: &[u8], b: &[u8], out: &mut [u8], seed: u32) -> usize {
    let (Ok(ta), Ok(tb)) = (std::str::from_utf8(a), std::str::from_utf8(b)) else {
        return copy_truncated(a, out);
    };
    let child = crossover_text(ta, tb, seed.into());
    copy_truncated(child.as_bytes(), out)
}

pub fn crossover_text(a: &str, b: &str, seed: u64) -> String {
    let mut rng = Rng::new(seed);
    let (ba, bb) = (boundaries(a), boundaries(b));
    if rng.below(2) == 0 {
        [&a[..pick(&mut rng, &ba)], &b[pick(&mut rng, &bb)..]].concat()
    } else {
        let (s, e) = token_range(&mut rng, &bb);
        let at = pick(&mut rng, &ba);
        splice(a, (at, at), &b[s..e])
    }
}

/// Cuts at a char boundary so a truncated child stays UTF-8.
fn copy_truncated(bytes: &[u8], out: &mut [u8]) -> usize {
    let mut n = bytes.len().min(out.len());
    while n > 0 && n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
        n -= 1;
    }
    out[..n].copy_from_slice(&bytes[..n]);
    n
}
