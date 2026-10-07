//! `ktrs_ktlint_version` (ktrs only, unknown to the real jars): which ktlint release the engine, the rules
//! and the `ktlint` drop-in imitate. Design and the list of switches: research/26-ktlint-18-mode.md.

use std::sync::LazyLock;

use ktrs_editorconfig::{EnumValue, PropertyType};

use crate::editorconfig::editor_config::EditorConfig;
use crate::editorconfig::editor_config_property::{EditorConfigProperty, PropertyRef};
use crate::engine::editor_config_defaults::EditorConfigOverride;
use crate::editorconfig::value_parsers::safe_enum_value_parser;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KtlintVersion {
    /// ktlint 1.8.0: lint rows and CLI behaviour (not yet its rule-major `--format` order).
    V1_8,
    /// ktlint 2.0.0-ALPHA-4, the port's base.
    #[default]
    V2_0,
}

impl EnumValue for KtlintVersion {
    const ENUM_TYPE_NAME: &'static str = "ktrs.KtlintVersion";
    const ENTRIES: &'static [Self] = &[KtlintVersion::V1_8, KtlintVersion::V2_0];

    fn name(self) -> &'static str {
        match self {
            KtlintVersion::V1_8 => "1.8",
            KtlintVersion::V2_0 => "2.0",
        }
    }
}

crate::enum_property_value_type!(KtlintVersion);

impl KtlintVersion {
    /// The file's version: `ktrs_ktlint_version` when set to a valid value, else 2.0. Read without an
    /// `EditorConfigProperty` lookup, so rules need not declare it.
    pub fn of(editor_config: &EditorConfig) -> KtlintVersion {
        editor_config
            .get_editor_config_value_or_null(&KTLINT_VERSION_PROPERTY_TYPE, KTLINT_VERSION_PROPERTY_TYPE.name)
            .unwrap_or_default()
    }

    pub fn is_1_8(self) -> bool {
        self == KtlintVersion::V1_8
    }
}

pub static KTLINT_VERSION_PROPERTY_TYPE: PropertyType<KtlintVersion> = PropertyType {
    name: "ktrs_ktlint_version",
    description: "The ktlint release ('1.8' or '2.0') whose lint results and CLI behaviour ktrs reproduces",
    parser: safe_enum_value_parser::<KtlintVersion>,
    possible_values: &["1.8", "2.0"],
    lower_casing: true,
};

pub static KTLINT_VERSION_PROPERTY: LazyLock<EditorConfigProperty<KtlintVersion>> =
    LazyLock::new(|| EditorConfigProperty::new(&KTLINT_VERSION_PROPERTY_TYPE, KtlintVersion::V2_0));

/// `editor_config_override` with `version` set, which every file's `.editorconfig` then reports.
pub fn with_ktlint_version(editor_config_override: EditorConfigOverride, version: KtlintVersion) -> EditorConfigOverride {
    let property = (PropertyRef::from(&*KTLINT_VERSION_PROPERTY), Some(version.name().to_owned()));
    if editor_config_override.is_empty() {
        EditorConfigOverride::from(vec![property])
    } else {
        editor_config_override.plus(vec![property])
    }
}
