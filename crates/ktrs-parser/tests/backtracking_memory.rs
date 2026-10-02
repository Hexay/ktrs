//! Type-argument backtracking is exponential in time (upstream too), but freed marker ids are
//! reused (`MarkerPool`), so peak memory stays bounded by the live markers (research/24, finding 1).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use ktrs_parser::{FileKind, parse_file};

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let live = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
        PEAK.fetch_max(live, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Peak heap growth while parsing `text`.
fn peak_parse_bytes(text: &str) -> usize {
    PEAK.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
    let base = LIVE.load(Ordering::Relaxed);
    let parse = parse_file(text, FileKind::Source);
    assert!(!parse.error_messages.is_empty(), "the input is meant to have parse errors");
    PEAK.load(Ordering::Relaxed) - base
}

// One test, so no other test's allocations run concurrently and skew the peak.
#[test]
fn nested_type_argument_backtracking_has_bounded_memory() {
    // Without marker reuse the first one peaks at 480 MB of heap; with it, 17 KB.
    for text in [format!("val x = {}", "(a<".repeat(12)), format!("val x = a<{}", "(".repeat(18))] {
        let peak = peak_parse_bytes(&text);
        assert!(peak < 4 << 20, "{text:?}: peak heap {} KB", peak >> 10);
    }
}
