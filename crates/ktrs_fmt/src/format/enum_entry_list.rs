//! Port of `EnumEntryList.kt` (lines 24-90): PSI-like model of a list of enum entries (KT-65157).

use ktrs_psi::{KtClass, KtClassBody, KtEnumEntry, PsiElement};

pub struct EnumEntryList {
    pub enum_entries: Vec<KtEnumEntry>,
    pub trailing_comma: Option<PsiElement>,
    pub terminating_semicolon: Option<PsiElement>,
}

impl EnumEntryList {
    pub fn extract_parent_list(enum_entry: &KtEnumEntry) -> EnumEntryList {
        let class_body = enum_entry.parent().and_then(|p| p.cast::<KtClassBody>()).expect("enum entry in a class body");
        Self::extract_child_list(&class_body).expect("enum entry in an enum class")
    }

    pub fn extract_child_list(class_body: &KtClassBody) -> Option<EnumEntryList> {
        let clazz = class_body.parent()?.cast::<KtClass>()?;
        if !clazz.is_enum() {
            return None;
        }

        let enum_entries: Vec<KtEnumEntry> = class_body.children().iter().filter_map(|c| c.cast()).collect();

        if enum_entries.is_empty() {
            let mut semicolon = class_body.first_child();
            while let Some(s) = &semicolon {
                if s.text() == ";" {
                    break;
                }
                semicolon = s.next_sibling();
            }

            return Some(EnumEntryList { enum_entries, trailing_comma: None, terminating_semicolon: semicolon });
        }

        let mut semicolon = None;
        let mut comma = None;
        let last_token = enum_entries
            .last()
            .unwrap()
            .last_child()
            .and_then(|c| c.get_prev_sibling_ignoring_whitespace_and_comments(true))
            .expect("enum entry has a non-comment child");
        match last_token.text().as_str() {
            "," => comma = Some(last_token),
            ";" => {
                let prev_sibling = last_token.get_prev_sibling_ignoring_whitespace_and_comments(false);
                if prev_sibling.as_ref().is_some_and(|p| p.text() == ",") {
                    comma = prev_sibling;
                }
                semicolon = Some(last_token);
            }
            _ => {}
        }

        Some(EnumEntryList { enum_entries, trailing_comma: comma, terminating_semicolon: semicolon })
    }
}
