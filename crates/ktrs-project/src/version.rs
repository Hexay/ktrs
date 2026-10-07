use crate::KtlintVersion;

/// The supported mode for a ktlint version string, and a note when it is not an exact match.
pub(crate) fn ktlint_version(raw: &str) -> (KtlintVersion, Option<String>) {
    let mut parts = raw.trim().split(['.', '-']);
    let major = parts.next().and_then(|p| p.parse::<u32>().ok());
    let minor = parts.next().and_then(|p| p.parse::<u32>().ok());
    match (major, minor) {
        (Some(1), Some(8)) => (KtlintVersion::V1_8, None),
        (Some(2), Some(0)) => (KtlintVersion::V2_0, None),
        (Some(m), _) if m >= 2 => {
            (KtlintVersion::V2_0, Some(format!("ktlint {raw} is not supported; using 2.0")))
        }
        (Some(_), _) => (KtlintVersion::V1_8, Some(format!("ktlint {raw} is not supported; using 1.8"))),
        (None, _) => (KtlintVersion::V1_8, Some(format!("ktlint version `{raw}` not understood; using 1.8"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_versions() {
        assert_eq!(ktlint_version("1.8.0"), (KtlintVersion::V1_8, None));
        assert_eq!(ktlint_version("2.0.0-ALPHA-4"), (KtlintVersion::V2_0, None));
        assert_eq!(ktlint_version("2.0.0-SNAPSHOT").0, KtlintVersion::V2_0);
        let (v, note) = ktlint_version("1.4.0");
        assert_eq!(v, KtlintVersion::V1_8);
        assert!(note.unwrap().contains("1.4.0"));
        assert_eq!(ktlint_version("0.50.0").0, KtlintVersion::V1_8);
        assert_eq!(ktlint_version("3.1").0, KtlintVersion::V2_0);
        assert!(ktlint_version("latest").1.is_some());
    }
}
