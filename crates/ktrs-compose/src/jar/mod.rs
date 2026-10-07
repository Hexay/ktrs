//! Recognizes a compose-rules (`io.nlopez.compose.rules`) ktlint JAR whose release ktrs runs natively
//! (`ktrs_compose`), by its contents: the `io/nlopez/compose/` entries' names, CRC-32s and sizes.

pub mod sha256;
pub mod zip_directory;

use std::path::Path;

use sha256::sha256;
use zip_directory::read_zip_directory;

/// The compose-rules release `jar` is, when ktrs ports it.
pub fn native_compose_rules_release(jar: &Path) -> Option<&'static str> {
    let fingerprint = fingerprint(&[jar])?;
    crate::NATIVE_JARS.iter().find(|(_, known)| *known == fingerprint).map(|(release, _)| *release)
}

/// SHA-256 of the sorted `name\tcrc32\tsize\n` lines of the `io/nlopez/compose/` file entries (the same as
/// `unzip -v <jar> | awk '$8 ~ /^io\/nlopez\/compose\/.*[^\/]$/ {printf "%s\t%s\t%s\n", $8, tolower($7), $1}' | LC_ALL=C sort | sha256sum`).
/// Directory entries don't count: JAR tools differ on them (the Gradle plugin's merged rule set JAR has none).
/// Over several JARs: their entries together, as one JAR merging them has them.
fn fingerprint(jars: &[&Path]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for jar in jars {
        lines.extend(
            read_zip_directory(jar)
                .ok()?
                .into_iter()
                .filter(|e| e.name.starts_with("io/nlopez/compose/") && !e.name.ends_with('/'))
                .map(|e| format!("{}\t{:08x}\t{}\n", e.name, e.crc32, e.size)),
        );
    }
    if lines.is_empty() {
        return None;
    }
    lines.sort();
    Some(sha256(lines.concat().as_bytes()).iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs `tools/sync-compose-rules.sh` (the JARs are not in git); skipped without it.
    #[test]
    fn pinned_release_jar_is_native() {
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/compose-rules/lib");
        let version = crate::COMPOSE_RULES_VERSION;
        let jar = lib.join(format!("ktlint-compose-{version}-all.jar"));
        if jar.is_file() {
            assert_eq!(native_compose_rules_release(&jar), Some(version));
        }
        assert_eq!(native_compose_rules_release(&lib.join("missing.jar")), None);
    }

    /// The Maven artifact and its `common-ktlint` dependency, merged into one JAR by the ktlint Gradle plugin.
    #[test]
    fn pinned_maven_artifacts_merged_are_native() {
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/compose-rules/lib");
        let version = crate::COMPOSE_RULES_VERSION;
        let (thin, common) = (lib.join(format!("ktlint-{version}.jar")), lib.join(format!("common-ktlint-{version}.jar")));
        if thin.is_file() && common.is_file() {
            let merged = fingerprint(&[&thin, &common]).unwrap();
            assert!(crate::NATIVE_JARS.contains(&(version, merged.as_str())), "{merged}");
            assert_eq!(native_compose_rules_release(&thin), None, "the thin JAR alone can't load");
        }
    }
}
