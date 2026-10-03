//! Recognizes a compose-rules (`io.nlopez.compose.rules`) ktlint JAR whose release ktrs runs natively
//! (`ktrs_compose`), by its contents: the `io/nlopez/compose/` entries' names, CRC-32s and sizes.

use std::path::Path;

use crate::ktlint::sha256::sha256;
use crate::ktlint::zip_directory::read_zip_directory;

/// The compose-rules release `jar` is, when ktrs ports it.
pub fn native_compose_rules_release(jar: &Path) -> Option<&'static str> {
    let fingerprint = fingerprint(jar)?;
    ktrs_compose::NATIVE_JARS.iter().find(|(_, known)| *known == fingerprint).map(|(release, _)| *release)
}

/// SHA-256 of the sorted `name\tcrc32\tsize\n` lines of the `io/nlopez/compose/` entries (the same as
/// `unzip -v <jar> | awk '$8 ~ /^io\/nlopez\/compose\// {printf "%s\t%s\t%s\n", $8, tolower($7), $1}' | LC_ALL=C sort | sha256sum`).
fn fingerprint(jar: &Path) -> Option<String> {
    let mut lines: Vec<String> = read_zip_directory(jar)
        .ok()?
        .into_iter()
        .filter(|e| e.name.starts_with("io/nlopez/compose/"))
        .map(|e| format!("{}\t{:08x}\t{}\n", e.name, e.crc32, e.size))
        .collect();
    if lines.is_empty() {
        return None;
    }
    lines.sort();
    Some(sha256(lines.concat().as_bytes()).iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs `tools/sync-compose-rules.sh` (the JAR is not in git); skipped without it.
    #[test]
    fn pinned_release_jar_is_native() {
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/compose-rules/lib");
        let jar = lib.join(format!("ktlint-compose-{}-all.jar", ktrs_compose::COMPOSE_RULES_VERSION));
        if jar.is_file() {
            assert_eq!(native_compose_rules_release(&jar), Some(ktrs_compose::COMPOSE_RULES_VERSION));
        }
        assert_eq!(native_compose_rules_release(&lib.join("missing.jar")), None);
    }
}
