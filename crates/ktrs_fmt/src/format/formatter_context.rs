//! Port of `FormatterContext.kt` (lines 21-29): parses lazily, re-parses only when a pass changed the code.

use std::cell::OnceCell;

use ktrs_psi::KtFile;

use super::FormatError;
use super::parser;

pub struct FormatterContext {
    pub code: String,
    kt_file: OnceCell<KtFile>,
}

impl FormatterContext {
    pub fn new(code: String) -> FormatterContext {
        FormatterContext { code, kt_file: OnceCell::new() }
    }

    fn kt_file(&self) -> Result<&KtFile, FormatError> {
        if self.kt_file.get().is_none() {
            let _ = self.kt_file.set(parser::parse(&self.code)?);
        }
        Ok(self.kt_file.get().unwrap())
    }

    pub fn transform(
        self,
        block: impl FnOnce(&KtFile) -> Result<String, FormatError>,
    ) -> Result<FormatterContext, FormatError> {
        let new_code = block(self.kt_file()?)?;
        Ok(if new_code == self.code { self } else { FormatterContext::new(new_code) })
    }
}
