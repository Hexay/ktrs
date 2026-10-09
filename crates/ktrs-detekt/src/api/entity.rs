//! `detekt-api/.../Entity.kt` and `Finding.kt`.

use ktrs_psi::{KtElement, KtFile, KtNamedDeclaration, PsiElement};

use super::location::Location;
use super::signatures::build_full_signature;

/// Stores information about a specific code fragment.
#[derive(Clone, Debug)]
pub struct Entity {
    pub signature: String,
    pub location: Location,
    pub kt_element: KtElement,
}

impl Entity {
    /// `Entity.from(element)`: everything from the element itself.
    pub fn from(element: &PsiElement) -> Entity {
        Entity::from_location(element, Location::from(element, 0))
    }

    /// `Entity.atName(element)`: at the name identifier of a named declaration.
    pub fn at_name(element: &PsiElement) -> Entity {
        let name_identifier = element.upcast::<KtNamedDeclaration>().name_identifier();
        Entity::from_for_signature(name_identifier.as_ref().unwrap_or(element), element)
    }

    /// `Entity.atPackageOrFirstDecl(file)`.
    pub fn at_package_or_first_decl(file: &KtFile) -> Entity {
        let element_to_report = file.package_directive().map(PsiElement::from).or_else(|| file.first_child());
        Entity::from_for_signature(element_to_report.as_ref().unwrap_or(file), file)
    }

    /// `Entity.from(element, location)`.
    pub fn from_location(element: &PsiElement, location: Location) -> Entity {
        Entity::from_all(element, element, location)
    }

    /// `Entity.from(elementToReport, elementForSignature)`.
    pub fn from_for_signature(element_to_report: &PsiElement, element_for_signature: &PsiElement) -> Entity {
        Entity::from_all(element_to_report, element_for_signature, Location::from(element_to_report, 0))
    }

    fn from_all(element_to_report: &PsiElement, element_for_signature: &PsiElement, location: Location) -> Entity {
        let signature = build_full_signature(element_for_signature);
        let kt_element = element_to_report.get_parent_of_type::<KtElement>(false).expect("KtElement expected");
        Entity { signature, location, kt_element }
    }
}

/// Represents a detected problem in the source code.
#[derive(Clone, Debug)]
pub struct Finding {
    pub entity: Entity,
    pub message: String,
    pub references: Vec<Entity>,
    pub suppress_reasons: Vec<String>,
}

impl Finding {
    pub fn new(entity: Entity, message: impl Into<String>) -> Finding {
        let message = message.into();
        assert!(!message.trim().is_empty(), "The message should not be empty");
        Finding { entity, message, references: Vec::new(), suppress_reasons: Vec::new() }
    }
}
