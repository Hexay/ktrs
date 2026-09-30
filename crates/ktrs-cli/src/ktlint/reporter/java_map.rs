//! A `java.util.concurrent.ConcurrentHashMap<String, V>` (default capacity, single-threaded use) that
//! iterates in the JVM's order: bins by index, nodes as `putVal`, `transfer` and tree bins leave them.
//! The html reporter iterates its map, so its file order depends on this.

const DEFAULT_CAPACITY: usize = 16;
const TREEIFY_THRESHOLD: usize = 8;
const UNTREEIFY_THRESHOLD: usize = 6;
const MIN_TREEIFY_CAPACITY: usize = 64;
const HASH_BITS: u32 = 0x7fff_ffff;

struct Bin<V> {
    nodes: Vec<(u32, String, V)>,
    /// A `TreeBin`: new nodes are prepended to its `first` list.
    tree: bool,
}

impl<V> Default for Bin<V> {
    fn default() -> Self {
        Bin { nodes: Vec::new(), tree: false }
    }
}

pub struct JavaConcurrentHashMap<V> {
    table: Vec<Bin<V>>,
    size: usize,
    size_ctl: usize,
}

impl<V> Default for JavaConcurrentHashMap<V> {
    fn default() -> Self {
        JavaConcurrentHashMap { table: Vec::new(), size: 0, size_ctl: 0 }
    }
}

/// `String.hashCode()` over UTF-16 code units.
pub fn java_string_hash(s: &str) -> i32 {
    s.encode_utf16().fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(c as i32))
}

fn spread(h: i32) -> u32 {
    let h = h as u32;
    (h ^ (h >> 16)) & HASH_BITS
}

fn table_size_for(c: usize) -> usize {
    c.max(1).next_power_of_two()
}

impl<V> JavaConcurrentHashMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn len(&self) -> usize {
        self.size
    }

    /// Kotlin's `getOrPut(key) { default() }` on a `ConcurrentMap` (`putIfAbsent` when missing).
    pub fn get_or_put(&mut self, key: &str, default: impl FnOnce() -> V) -> &mut V {
        let hash = spread(java_string_hash(key));
        if let Some((bin, pos)) = self.find(hash, key) {
            return &mut self.table[bin].nodes[pos].2;
        }
        self.put_val(hash, key.to_owned(), default());
        let (bin, pos) = self.find(hash, key).expect("just inserted");
        &mut self.table[bin].nodes[pos].2
    }

    fn find(&self, hash: u32, key: &str) -> Option<(usize, usize)> {
        if self.table.is_empty() {
            return None;
        }
        let i = hash as usize & (self.table.len() - 1);
        self.table[i].nodes.iter().position(|(h, k, _)| *h == hash && k == key).map(|pos| (i, pos))
    }

    fn put_val(&mut self, hash: u32, key: String, value: V) {
        if self.table.is_empty() {
            self.table = (0..DEFAULT_CAPACITY).map(|_| Bin::default()).collect();
            self.size_ctl = DEFAULT_CAPACITY - (DEFAULT_CAPACITY >> 2);
        }
        let n = self.table.len();
        let i = hash as usize & (n - 1);
        let bin = &mut self.table[i];
        let bin_count = if bin.tree {
            bin.nodes.insert(0, (hash, key, value));
            2
        } else {
            let count = bin.nodes.len();
            bin.nodes.push((hash, key, value));
            count
        };
        if bin_count >= TREEIFY_THRESHOLD {
            self.treeify_bin(i);
        }
        self.add_count();
    }

    fn treeify_bin(&mut self, index: usize) {
        let n = self.table.len();
        if n < MIN_TREEIFY_CAPACITY {
            self.try_presize(n << 1);
        } else {
            self.table[index].tree = true;
        }
    }

    fn try_presize(&mut self, size: usize) {
        let c = table_size_for(size + (size >> 1) + 1);
        while c > self.size_ctl {
            self.transfer();
        }
    }

    fn add_count(&mut self) {
        self.size += 1;
        while self.size >= self.size_ctl {
            self.transfer();
        }
    }

    fn transfer(&mut self) {
        let n = self.table.len();
        let mut next: Vec<Bin<V>> = (0..n << 1).map(|_| Bin::default()).collect();
        for (i, bin) in std::mem::take(&mut self.table).into_iter().enumerate() {
            if bin.nodes.is_empty() {
                continue;
            }
            let (lo, hi) = if bin.tree { split_tree(bin.nodes, n) } else { split_list(bin.nodes, n) };
            next[i] = lo;
            next[i + n] = hi;
        }
        self.table = next;
        self.size_ctl = (n << 1) - (n >> 1);
    }
}

/// `transfer` of a list bin: the tail run with the same bit stays intact, the nodes before it are
/// prepended one by one (so their order reverses).
fn split_list<V>(nodes: Vec<(u32, String, V)>, n: usize) -> (Bin<V>, Bin<V>) {
    let bit = |h: u32| h as usize & n;
    let mut run_bit = bit(nodes[0].0);
    let mut last_run = 0;
    for (p, node) in nodes.iter().enumerate().skip(1) {
        if bit(node.0) != run_bit {
            run_bit = bit(node.0);
            last_run = p;
        }
    }
    let mut nodes = nodes;
    let tail = nodes.split_off(last_run);
    let (mut ln, mut hn) = if run_bit == 0 { (tail, Vec::new()) } else { (Vec::new(), tail) };
    for node in nodes {
        if bit(node.0) == 0 { ln.insert(0, node) } else { hn.insert(0, node) }
    }
    (Bin { nodes: ln, tree: false }, Bin { nodes: hn, tree: false })
}

/// `transfer` of a `TreeBin`: order kept; a half with at most `UNTREEIFY_THRESHOLD` nodes becomes a list.
fn split_tree<V>(nodes: Vec<(u32, String, V)>, n: usize) -> (Bin<V>, Bin<V>) {
    let (lo, hi): (Vec<_>, Vec<_>) = nodes.into_iter().partition(|node| node.0 as usize & n == 0);
    let tree = |nodes: &Vec<(u32, String, V)>| nodes.len() > UNTREEIFY_THRESHOLD;
    (Bin { tree: tree(&lo), nodes: lo }, Bin { tree: tree(&hi), nodes: hi })
}

impl<V> JavaConcurrentHashMap<V> {
    /// `entries` / `forEach` order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.table.iter().flat_map(|bin| bin.nodes.iter().map(|(_, k, v)| (k.as_str(), v)))
    }
}
