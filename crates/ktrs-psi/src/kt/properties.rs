//! `KtProperty`, `KtPropertyAccessor`, `KtBackingField`, `KtPropertyDelegate`, `KtParameter`,
//! `KtParameterList`, `KtDestructuringDeclaration(Entry)`.

use ktrs_syntax::SyntaxKind::*;

use super::declarations::expression_after_eq;
use crate::element::PsiElement;
use crate::tokens::{CLOSING_BRACES, OPENING_BRACES, VAL_VAR};
use crate::tree_util::{get_next_sibling_of_type, get_trailing_comma_by_closing_element, get_trailing_comma_by_elements_list};
use crate::types::*;

impl KtProperty {
    /// `getValOrVarKeyword()`; upstream asserts non-null, which is off at runtime.
    pub fn val_or_var_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type_set(VAL_VAR)
    }

    pub fn delegate(&self) -> Option<KtPropertyDelegate> {
        self.find_child_by_type(PROPERTY_DELEGATE)
    }

    pub fn delegate_expression(&self) -> Option<KtExpression> {
        self.delegate()?.expression()
    }

    pub fn initializer(&self) -> Option<KtExpression> {
        expression_after_eq(self)
    }

    pub fn accessors(&self) -> Vec<KtPropertyAccessor> {
        self.get_stub_or_psi_children(PROPERTY_ACCESSOR)
    }

    pub fn getter(&self) -> Option<KtPropertyAccessor> {
        self.accessors().into_iter().find(KtPropertyAccessor::is_getter)
    }

    pub fn setter(&self) -> Option<KtPropertyAccessor> {
        self.accessors().into_iter().find(KtPropertyAccessor::is_setter)
    }

    pub fn field_declaration(&self) -> Option<KtBackingField> {
        self.get_stub_or_psi_children(BACKING_FIELD).into_iter().next()
    }

    pub fn is_var(&self) -> bool {
        self.node().find_child_by_type(VAR_KEYWORD).is_some()
    }
}

impl KtPropertyAccessor {
    pub fn is_getter(&self) -> bool {
        self.find_child_by_type::<PsiElement>(GET_KEYWORD).is_some()
    }

    pub fn is_setter(&self) -> bool {
        self.find_child_by_type::<PsiElement>(SET_KEYWORD).is_some()
    }

    pub fn parameter_list(&self) -> Option<KtParameterList> {
        self.get_stub_or_psi_child(VALUE_PARAMETER_LIST)
    }

    pub fn parameter(&self) -> Option<KtParameter> {
        self.parameter_list()?.parameters().into_iter().next()
    }

    /// `getNamePlaceholder()`: `get`/`set` keyword; upstream throws when neither exists (None here).
    pub fn name_placeholder(&self) -> Option<PsiElement> {
        self.find_child_by_type(GET_KEYWORD).or_else(|| self.find_child_by_type(SET_KEYWORD))
    }

    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }

    pub fn return_type_reference(&self) -> Option<KtTypeReference> {
        self.type_reference()
    }

    pub fn equals_token(&self) -> Option<PsiElement> {
        self.find_child_by_type(EQ)
    }

    pub fn initializer(&self) -> Option<KtExpression> {
        get_next_sibling_of_type(self.equals_token().as_ref())
    }

    pub fn left_parenthesis(&self) -> Option<PsiElement> {
        self.parameter_list()?.left_parenthesis()
    }

    pub fn right_parenthesis(&self) -> Option<PsiElement> {
        self.parameter_list()?.right_parenthesis()
    }
}

impl KtBackingField {
    pub fn equals_token(&self) -> Option<PsiElement> {
        self.find_child_by_type(EQ)
    }

    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }

    /// `getNamePlaceholder()`: the `field` keyword, else the backing field itself.
    pub fn name_placeholder(&self) -> PsiElement {
        self.field_keyword().unwrap_or_else(|| PsiElement::clone(self))
    }

    pub fn initializer(&self) -> Option<KtExpression> {
        get_next_sibling_of_type(self.equals_token().as_ref())
    }

    pub fn field_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type(FIELD_KEYWORD)
    }

    pub fn return_type_reference(&self) -> Option<KtTypeReference> {
        self.type_reference()
    }
}

impl KtPropertyDelegate {
    pub fn expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtParameter {
    pub fn equals_token(&self) -> Option<PsiElement> {
        self.find_child_by_type(EQ)
    }

    pub fn default_value(&self) -> Option<KtExpression> {
        let equals_token = self.equals_token()?;
        get_next_sibling_of_type(Some(&equals_token))
    }

    pub fn val_or_var_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type_set(VAL_VAR)
    }

    pub fn destructuring_declaration(&self) -> Option<KtDestructuringDeclaration> {
        self.find_child_by_type(DESTRUCTURING_DECLARATION)
    }
}

impl KtParameterList {
    pub fn parameters(&self) -> Vec<KtParameter> {
        self.get_stub_or_psi_children(VALUE_PARAMETER)
    }

    pub fn right_parenthesis(&self) -> Option<PsiElement> {
        self.find_child_by_type(RPAR)
    }

    pub fn left_parenthesis(&self) -> Option<PsiElement> {
        self.find_child_by_type(LPAR)
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        if self.parent().is_some_and(|p| p.is::<KtFunctionLiteral>()) {
            get_trailing_comma_by_elements_list(Some(self))
        } else {
            get_trailing_comma_by_closing_element(self.right_parenthesis().as_ref())
        }
    }
}

impl KtDestructuringDeclaration {
    /// `getEntries()`: entries, including those the parser wrapped in an error element.
    pub fn entries(&self) -> Vec<KtDestructuringDeclarationEntry> {
        let mut result = Vec::new();
        for child in self.all_children() {
            if child.kind() == DESTRUCTURING_DECLARATION_ENTRY {
                result.push(child.upcast());
            } else if child.kind() == ERROR_ELEMENT {
                result.extend(child.all_children().filter(|c| c.kind() == DESTRUCTURING_DECLARATION_ENTRY).map(|c| c.upcast()));
            }
        }
        result
    }

    pub fn initializer(&self) -> Option<KtExpression> {
        let eq = self.node().find_child_by_type(EQ)?;
        get_next_sibling_of_type(Some(&eq.psi()))
    }

    pub fn val_or_var_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type_set(VAL_VAR)
    }

    pub fn r_par(&self) -> Option<PsiElement> {
        self.find_child_by_type_set(CLOSING_BRACES)
    }

    pub fn l_par(&self) -> Option<PsiElement> {
        self.find_child_by_type_set(OPENING_BRACES)
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        get_trailing_comma_by_closing_element(self.r_par().as_ref())
    }
}

impl KtDestructuringDeclarationEntry {
    pub fn equals_token(&self) -> Option<PsiElement> {
        self.find_child_by_type(EQ)
    }

    pub fn initializer(&self) -> Option<KtNameReferenceExpression> {
        get_next_sibling_of_type(self.equals_token().as_ref())
    }

    /// `getValOrVarKeyword()`: the keyword of the enclosing destructuring declaration.
    pub fn val_or_var_keyword(&self) -> Option<PsiElement> {
        let mut parent = self.parent()?;
        if parent.kind() == ERROR_ELEMENT {
            parent = parent.parent()?;
        }
        parent.find_child_by_type_set(VAL_VAR)
    }

    pub fn own_val_or_var_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type_set(VAL_VAR)
    }
}
