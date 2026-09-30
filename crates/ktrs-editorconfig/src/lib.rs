//! Port of ec4j-core 1.2.0 (`org.ec4j.core`), the EditorConfig implementation ktlint (and ktfmt) run on,
//! decompiled from the jar ktlint 2.0.0-ALPHA-4 pins. ktlint depends on its exact quirks: values keep their
//! case (only names are lowercased), `indent_size`/`tab_width` defaults are derived per section at parse
//! time, `unset` values can be kept, and globs are compiled to (Java) regular expressions.
//!
//! # Porting conventions
//! - One Rust fn per Java method, `snake_case`, in file order; Java builders collapse into plain structs.
//! - `Resource`/`ResourcePath` are file-system paths; `Ec4jPath` is a `/`-separated relative path string.
//! - Exceptions are `Err(ParseException)`; `IOException`s become `ParseException`s of type `Other`.

pub mod editor_config;
mod glob;
mod parser;
pub mod property_type;
mod resource_properties_service;

pub use editor_config::{EditorConfig, Property, Section};
pub use glob::Glob;
pub use parser::{ErrorType, ParseException, PropertyTypeRegistry, load, parse};
pub use property_type::{
    AnyPropertyType, EndOfLineValue, EnumValue, IndentStyleValue, PropertyType, PropertyValue,
};
pub use resource_properties_service::{
    Cache, NoCache, ResourceProperties, ResourcePropertiesService,
};
