//! `cargo run -p ktrs-project --example detect -- <file or dir>...`: what `detect` finds for each path.

use std::path::Path;

fn main() {
    for arg in std::env::args().skip(1) {
        let c = ktrs_project::detect(Path::new(&arg));
        println!("{arg}\n  root:   {}\n  format: {:?}\n  ktlint: {:?}", c.root.display(), c.format, c.ktlint);
        for note in &c.notes {
            println!("  - {note}");
        }
    }
}
