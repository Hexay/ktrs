//! Accessors, grouped by PSI class family. Methods declared on an upstream interface/abstract class are
//! stamped onto every PSI type that has them (so abstract and concrete views expose the same method) and
//! resolve the upstream override by element type, like Java's virtual dispatch.

mod annotations;
mod calls;
mod classes;
mod control;
mod declarations;
mod file;
mod import_path;
mod kdoc;
mod operators;
mod properties;
mod type_refs;

pub use annotations::AnnotationUseSiteTarget;
pub use calls::{unquote_identifier, unquote_identifier_or_field_reference};
pub use file::{FqName, ImportPath, KtFile};
pub use import_path::render_name;
pub use type_refs::KtProjectionKind;
