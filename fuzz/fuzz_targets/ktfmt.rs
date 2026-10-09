//! ktfmt port: no panic on any input, all three styles. Java-exception panics are ktfmt's own exceptions
//! (see `ktrs_fuzz::is_java_exception`); every other panic is a finding.
#![no_main]

use ktrs_fmt::{FileType, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT};
use libfuzzer_sys::fuzz_target;

ktrs_fuzz::splice_mutators!();

fuzz_target!(|data: &[u8]| {
    let Some(text) = ktrs_fuzz::fuzz_input(data) else { return };
    for options in [&META_FORMAT, &GOOGLE_FORMAT, &KOTLINLANG_FORMAT] {
        for file_type in [FileType::Regular, FileType::Script] {
            let _ = ktrs_fuzz::tolerate_java_panics(|| ktrs_fmt::format(text, file_type, options));
        }
    }
});
