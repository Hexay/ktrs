//! The subset of Guava's `ImmutableRangeMap<Integer, V>` over closed ranges that gjf uses.

#[derive(Clone, Debug)]
pub struct RangeMap<V> {
    /// Closed `[lower, upper]` entries, sorted and non-overlapping.
    entries: Vec<(i32, i32, V)>,
}

impl<V> RangeMap<V> {
    /// `ImmutableRangeMap.builder()...build()` from closed ranges. Panics on overlap, like Guava.
    pub fn from_closed(mut entries: Vec<(i32, i32, V)>) -> RangeMap<V> {
        entries.sort_by_key(|e| e.0);
        for w in entries.windows(2) {
            assert!(
                w[0].1 < w[1].0,
                "Overlapping ranges: [{}..{}] and [{}..{}]",
                w[0].0,
                w[0].1,
                w[1].0,
                w[1].1
            );
        }
        RangeMap { entries }
    }

    /// `get(key)`.
    pub fn get(&self, key: i32) -> Option<&V> {
        let at = self.entries.partition_point(|e| e.1 < key);
        self.entries.get(at).filter(|e| e.0 <= key).map(|e| &e.2)
    }

    /// `subRangeMap(Range.closedOpen(lower, upper)).asMapOfRanges().values()`.
    pub fn sub_range_values_closed_open(&self, lower: i32, upper: i32) -> Vec<&V> {
        self.entries
            .iter()
            .filter(|e| e.0 < upper && lower <= e.1)
            .map(|e| &e.2)
            .collect()
    }
}
