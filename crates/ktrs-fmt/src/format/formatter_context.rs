//! Port of `FormatterContext.kt` (lines 21-29): parses lazily, re-parses only when a pass changed the code.
//! Deviation: the contexts of one `format` call share a [`ChameleonCache`], so a re-parse reuses
//! every block and lambda whose text the pass left alone (the tree is identical either way).

use std::cell::{OnceCell, RefCell};

use ktrs_parser::ChameleonCache;
use ktrs_psi::KtFile;

use super::FormatError;
use super::kotlin_code::KotlinCode;
use super::parser;

pub struct FormatterContext {
    pub code: KotlinCode,
    kt_file: OnceCell<KtFile>,
    cache: RefCell<ChameleonCache>,
}

impl FormatterContext {
    pub fn new(code: KotlinCode) -> FormatterContext {
        FormatterContext { code, kt_file: OnceCell::new(), cache: RefCell::default() }
    }

    fn kt_file(&self) -> Result<&KtFile, FormatError> {
        if self.kt_file.get().is_none() {
            let _ = self.kt_file.set(parser::parse(&self.code, &mut self.cache.borrow_mut())?);
        }
        Ok(self.kt_file.get().unwrap())
    }

    pub fn transform(
        self,
        block: impl FnOnce(&KtFile) -> Result<String, FormatError>,
    ) -> Result<FormatterContext, FormatError> {
        Ok(self.transform_changed(block)?.0)
    }

    /// [`Self::transform`], also telling whether it returned a new context (upstream compares identities).
    pub fn transform_changed(
        self,
        block: impl FnOnce(&KtFile) -> Result<String, FormatError>,
    ) -> Result<(FormatterContext, bool), FormatError> {
        let new_code = block(self.kt_file()?)?;
        Ok(if new_code == self.code.code {
            (self, false)
        } else {
            (FormatterContext { code: self.code.copy(new_code), kt_file: OnceCell::new(), cache: self.cache }, true)
        })
    }
}
