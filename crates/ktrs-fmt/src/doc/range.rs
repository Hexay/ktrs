//! The subset of Guava's `Range<Integer>` and `TreeRangeSet<Integer>` that gjf uses.
//!
//! Every `Range` gjf builds for docs/lines canonicalizes to `[lower, upper)`, so [`Range`] stores
//! that form. `RangeSet` keeps real bound types because Guava only coalesces *connected* ranges:
//! closed `[0, 2]` and `[3, 5]` stay apart, while their canonical forms `[0, 3)`, `[3, 6)` merge.

/// A canonical integer range `[lower, upper)`; empty iff `lower == upper`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Range {
    lower: i32,
    upper: i32,
}

/// `Range.closedOpen(-1, -1)`.
pub const EMPTY_RANGE: Range = Range {
    lower: -1,
    upper: -1,
};

impl Range {
    pub fn closed_open(lower: i32, upper: i32) -> Range {
        assert!(lower <= upper, "Invalid range: [{lower}..{upper})");
        Range { lower, upper }
    }

    /// `Range.singleton(k).canonical(integers())`.
    pub fn singleton(k: i32) -> Range {
        Range {
            lower: k,
            upper: k + 1,
        }
    }

    pub fn is_empty(self) -> bool {
        self.lower == self.upper
    }

    pub fn lower_endpoint(self) -> i32 {
        self.lower
    }

    pub fn upper_endpoint(self) -> i32 {
        self.upper
    }

    pub fn span(self, other: Range) -> Range {
        Range {
            lower: self.lower.min(other.lower),
            upper: self.upper.max(other.upper),
        }
    }

    pub fn contains(self, k: i32) -> bool {
        self.lower <= k && k < self.upper
    }
}

/// A Guava `Cut` over integers: `(value, false)` is `belowValue`, `(value, true)` is `aboveValue`.
type Cut = (i32, bool);

/// A Guava `Range<Integer>` with a closed lower bound and either upper bound type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BoundedRange {
    lower: Cut,
    upper: Cut,
}

impl BoundedRange {
    fn is_connected(self, other: BoundedRange) -> bool {
        self.lower <= other.upper && other.lower <= self.upper
    }

    fn contains(self, k: i32) -> bool {
        self.lower <= (k, false) && (k, false) < self.upper
    }

    fn lower_endpoint(self) -> i32 {
        self.lower.0
    }

    fn upper_endpoint(self) -> i32 {
        self.upper.0
    }

    fn canonical(self) -> Range {
        Range {
            lower: self.lower.0,
            upper: if self.upper.1 {
                self.upper.0 + 1
            } else {
                self.upper.0
            },
        }
    }
}

/// `TreeRangeSet<Integer>`: disjoint, unconnected ranges sorted by lower bound.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RangeSet {
    ranges: Vec<BoundedRange>,
}

impl RangeSet {
    pub fn create() -> RangeSet {
        RangeSet::default()
    }

    /// `add(Range.closedOpen(lower, upper))`.
    pub fn add(&mut self, range: Range) {
        self.add_bounded(BoundedRange {
            lower: (range.lower, false),
            upper: (range.upper, false),
        });
    }

    /// `add(Range.closed(lower, upper))`.
    pub fn add_closed(&mut self, lower: i32, upper: i32) {
        assert!(lower <= upper, "Invalid range: [{lower}..{upper}]");
        self.add_bounded(BoundedRange {
            lower: (lower, false),
            upper: (upper, true),
        });
    }

    fn add_bounded(&mut self, mut range: BoundedRange) {
        if range.lower == range.upper {
            return;
        }
        // The stored ranges are sorted and pairwise unconnected, so the ones connected to `range`
        // are a contiguous run: log-time search, as in Guava's TreeRangeSet.
        let start = self.ranges.partition_point(|r| r.upper < range.lower);
        let end = self.ranges.partition_point(|r| r.lower <= range.upper).max(start);
        if start < end {
            range = BoundedRange {
                lower: self.ranges[start].lower.min(range.lower),
                upper: self.ranges[end - 1].upper.max(range.upper),
            };
        }
        self.ranges.splice(start..end, [range]);
    }

    pub fn contains(&self, k: i32) -> bool {
        self.range_index_containing(k).is_some()
    }

    /// `rangeContaining(k)` as `(lowerEndpoint, upperEndpoint)`.
    pub fn range_containing(&self, k: i32) -> Option<(i32, i32)> {
        self.range_index_containing(k)
            .map(|i| (self.ranges[i].lower_endpoint(), self.ranges[i].upper_endpoint()))
    }

    fn range_index_containing(&self, k: i32) -> Option<usize> {
        let i = self.ranges.partition_point(|r| r.upper <= (k, false));
        self.ranges.get(i).is_some_and(|r| r.contains(k)).then_some(i)
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// `addAll(other)`.
    pub fn add_all(&mut self, other: &RangeSet) {
        for range in &other.ranges {
            self.add_bounded(*range);
        }
    }

    /// `subRangeSet(Range.closed(lower, upper))`.
    pub fn sub_range_set_closed(&self, lower: i32, upper: i32) -> RangeSet {
        self.sub_range_set_bounded(BoundedRange {
            lower: (lower, false),
            upper: (upper, true),
        })
    }

    /// `subRangeSet(Range.closedOpen(lower, upper))`.
    pub fn sub_range_set(&self, range: Range) -> RangeSet {
        self.sub_range_set_bounded(BoundedRange {
            lower: (range.lower, false),
            upper: (range.upper, false),
        })
    }

    fn sub_range_set_bounded(&self, bound: BoundedRange) -> RangeSet {
        let ranges = self
            .ranges
            .iter()
            .filter(|r| r.is_connected(bound))
            .map(|r| BoundedRange {
                lower: r.lower.max(bound.lower),
                upper: r.upper.min(bound.upper),
            })
            .filter(|r| r.lower != r.upper)
            .collect();
        RangeSet { ranges }
    }

    /// `asRanges()`, each `canonical(integers())`.
    pub fn as_canonical_ranges(&self) -> Vec<Range> {
        self.ranges.iter().map(|r| r.canonical()).collect()
    }
}
