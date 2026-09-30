//! Ports of ktlint-rule-engine `api/EditorConfigDefaults.kt`, `internal/EditorConfigDefaultsLoader.kt` and
//! `api/EditorConfigOverride.kt`.

use std::path::Path;
use std::sync::Arc;

use ktrs_editorconfig::{
    AnyPropertyType, Cache, EditorConfig as Ec4jEditorConfig, ParseException, PropertyTypeRegistry,
    PropertyValue,
};

use crate::editorconfig::{EditorConfigProperty, PropertyRef, PropertyValueType};
use crate::engine::editor_config_cache::THREAD_SAFE_EDITOR_CONFIG_CACHE;

/// Defaults for properties that no `.editorconfig` on the path sets: an ec4j config whose sections apply
/// as if it sat in the farthest directory that was searched.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorConfigDefaults {
    pub value: Arc<Ec4jEditorConfig>,
}

impl EditorConfigDefaults {
    /// `EMPTY_EDITOR_CONFIG_DEFAULTS`.
    pub fn empty() -> EditorConfigDefaults {
        EditorConfigDefaults::default()
    }

    /// `EditorConfigDefaults.load(path, propertyTypes)` (`EditorConfigDefaultsLoader.load`): `path` is an
    /// `.editorconfig`-format file (any name) or a directory containing `.editorconfig`; `root` is ignored.
    pub fn load(
        path: Option<&Path>,
        property_types: &[&'static dyn AnyPropertyType],
    ) -> Result<EditorConfigDefaults, ParseException> {
        let Some(path) = path.filter(|p| !p.to_string_lossy().trim().is_empty()) else {
            return Ok(EditorConfigDefaults::empty());
        };
        let editor_config_file_path = if path.is_dir() {
            path.join(".editorconfig")
        } else {
            path.to_path_buf()
        };
        if !editor_config_file_path.exists() {
            return Ok(EditorConfigDefaults::empty());
        }
        let registry = PropertyTypeRegistry::with_defaults(property_types.iter().copied());
        Ok(EditorConfigDefaults {
            value: THREAD_SAFE_EDITOR_CONFIG_CACHE.get(&editor_config_file_path, &registry)?,
        })
    }
}

/// Values that replace whatever the `.editorconfig` files say (API consumers, unit tests).
#[derive(Clone, Debug, Default)]
pub struct EditorConfigOverride {
    properties: Vec<(PropertyRef, PropertyValue<()>)>,
}

impl EditorConfigOverride {
    /// `EMPTY_EDITOR_CONFIG_OVERRIDE`.
    pub fn empty() -> EditorConfigOverride {
        EditorConfigOverride::default()
    }

    pub fn properties(&self) -> &[(PropertyRef, PropertyValue<()>)] {
        &self.properties
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// The value set for `property` (keys compare like `@Poko` properties).
    pub fn get(&self, property: &PropertyRef) -> Option<&PropertyValue<()>> {
        self.properties
            .iter()
            .find(|(p, _)| p.identity() == property.identity())
            .map(|(_, v)| v)
    }

    /// `add(property, value)` with `value?.toString()`: parsed by the property's type.
    fn add(&mut self, property: PropertyRef, value: Option<&str>) {
        let parsed = property.parse(value);
        match self
            .properties
            .iter_mut()
            .find(|(p, _)| p.identity() == property.identity())
        {
            Some(entry) => entry.1 = parsed,
            None => self.properties.push((property, parsed)),
        }
    }

    /// `EditorConfigOverride.from(property to value, ...)` with the values as their `toString()`.
    pub fn from(properties: Vec<(PropertyRef, Option<String>)>) -> EditorConfigOverride {
        assert!(
            !properties.is_empty(),
            "IllegalArgumentException: Can not create an EditorConfigOverride without properties. Use 'emptyEditorConfigOverride' instead."
        );
        let mut editor_config_override = EditorConfigOverride::default();
        properties
            .into_iter()
            .for_each(|(p, v)| editor_config_override.add(p, v.as_deref()));
        editor_config_override
    }

    /// `plus(property to value, ...)`: a copy with the given values set (replacing existing ones).
    pub fn plus(&self, properties: Vec<(PropertyRef, Option<String>)>) -> EditorConfigOverride {
        assert!(
            !properties.is_empty(),
            "IllegalArgumentException: Can not add EditorConfigOverride without properties."
        );
        let mut editor_config_override = self.clone();
        properties
            .into_iter()
            .for_each(|(p, v)| editor_config_override.add(p, v.as_deref()));
        editor_config_override
    }

    /// Typed convenience: `plus(property to value)`.
    pub fn with<T: PropertyValueType>(
        &self,
        property: &'static EditorConfigProperty<T>,
        value: T,
    ) -> EditorConfigOverride {
        let mut editor_config_override = self.clone();
        editor_config_override.add(property.into(), Some(&value.to_value_string()));
        editor_config_override
    }
}
