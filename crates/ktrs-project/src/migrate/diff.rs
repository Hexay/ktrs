//! A line-based unified diff (3 lines of context), enough for build files.

const CONTEXT: usize = 3;
/// Above this many line pairs, skip the LCS table and show the whole file replaced.
const MAX_CELLS: usize = 16_000_000;

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Same,
    Del,
    Add,
}

/// `--- a/<label>` / `+++ b/<label>` and hunks; empty when the texts are equal.
pub fn unified(label: &str, old: &str, new: &str) -> String {
    if old == new {
        return String::new();
    }
    let a: Vec<&str> = old.split_inclusive('\n').collect();
    let b: Vec<&str> = new.split_inclusive('\n').collect();
    let ops = script(&a, &b);
    let mut out = format!("--- a/{label}\n+++ b/{label}\n");
    let changed: Vec<usize> = (0..ops.len()).filter(|&k| ops[k].0 != Op::Same).collect();
    let mut k = 0;
    while k < changed.len() {
        let start = changed[k].saturating_sub(CONTEXT);
        let mut end = changed[k] + 1;
        while k + 1 < changed.len() && changed[k + 1] <= end + 2 * CONTEXT {
            k += 1;
            end = changed[k] + 1;
        }
        let end = (end + CONTEXT).min(ops.len());
        hunk(&ops[start..end], &a, &b, &mut out);
        k += 1;
    }
    out
}

/// Each op with the line index it consumes in `a` (Same, Del) or `b` (Add), and the position in the other.
fn script(a: &[&str], b: &[&str]) -> Vec<(Op, usize, usize)> {
    let prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..].iter().rev().zip(b[prefix..].iter().rev()).take_while(|(x, y)| x == y).count();
    let (am, bm) = (&a[prefix..a.len() - suffix], &b[prefix..b.len() - suffix]);
    let mut ops: Vec<(Op, usize, usize)> = (0..prefix).map(|i| (Op::Same, i, i)).collect();
    if am.len().saturating_mul(bm.len()) > MAX_CELLS {
        ops.extend((0..am.len()).map(|i| (Op::Del, prefix + i, prefix)));
        ops.extend((0..bm.len()).map(|j| (Op::Add, prefix + am.len(), prefix + j)));
    } else {
        let (n, m) = (am.len(), bm.len());
        let mut lcs = vec![0u32; (n + 1) * (m + 1)];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                lcs[i * (m + 1) + j] = if am[i] == bm[j] {
                    lcs[(i + 1) * (m + 1) + j + 1] + 1
                } else {
                    lcs[(i + 1) * (m + 1) + j].max(lcs[i * (m + 1) + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < n || j < m {
            if i < n && j < m && am[i] == bm[j] {
                ops.push((Op::Same, prefix + i, prefix + j));
                i += 1;
                j += 1;
            } else if i < n && (j == m || lcs[(i + 1) * (m + 1) + j] >= lcs[i * (m + 1) + j + 1]) {
                ops.push((Op::Del, prefix + i, prefix + j));
                i += 1;
            } else {
                ops.push((Op::Add, prefix + i, prefix + j));
                j += 1;
            }
        }
    }
    let (sa, sb) = (a.len() - suffix, b.len() - suffix);
    ops.extend((0..suffix).map(|k| (Op::Same, sa + k, sb + k)));
    ops
}

fn hunk(ops: &[(Op, usize, usize)], a: &[&str], b: &[&str], out: &mut String) {
    let a_start = ops[0].1;
    let b_start = ops[0].2;
    let a_len = ops.iter().filter(|o| o.0 != Op::Add).count();
    let b_len = ops.iter().filter(|o| o.0 != Op::Del).count();
    let range = |start: usize, len: usize| if len == 0 { format!("{start},0") } else { format!("{},{len}", start + 1) };
    out.push_str(&format!("@@ -{} +{} @@\n", range(a_start, a_len), range(b_start, b_len)));
    for &(op, i, j) in ops {
        let (sign, line) = match op {
            Op::Same => (' ', a[i]),
            Op::Del => ('-', a[i]),
            Op::Add => ('+', b[j]),
        };
        out.push(sign);
        out.push_str(line);
        if !line.ends_with('\n') {
            out.push_str("\n\\ No newline at end of file\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::unified;

    #[test]
    fn hunks() {
        let old = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n";
        let new = "a\nB\nc\nd\ne\nf\ng\nh\ni\nj\nk\n";
        assert_eq!(
            unified("f", old, new),
            "--- a/f\n+++ b/f\n@@ -1,5 +1,5 @@\n a\n-b\n+B\n c\n d\n e\n@@ -8,3 +8,4 @@\n h\n i\n j\n+k\n"
        );
        assert_eq!(unified("f", "x\n", "x\n"), "");
        assert_eq!(unified("f", "", "x\n"), "--- a/f\n+++ b/f\n@@ -0,0 +1,1 @@\n+x\n");
    }
}
