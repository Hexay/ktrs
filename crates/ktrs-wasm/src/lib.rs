//! The formatter as a WebAssembly module with plain exports (no wasm-bindgen), for site/: JS copies
//! UTF-8 code into a buffer from [`alloc`], calls [`format`] (which takes that buffer), reads the
//! result it returns, `[status u8][length u32 LE][UTF-8]`, and frees it with [`dealloc`].
//! Build: `tools/release/build-site.sh`.

use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT};

pub const STATUS_OK: u8 = 0;
pub const STATUS_ERROR: u8 = 1;

/// A buffer of `len` bytes for the caller to fill.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buffer = std::mem::ManuallyDrop::new(Vec::<u8>::with_capacity(len));
    buffer.as_mut_ptr()
}

/// Frees a buffer from [`alloc`] or [`format`].
///
/// # Safety
/// `ptr` and `len` must be a buffer from [`alloc`] (its `len`) or [`format`] (5 + its length field).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(unsafe { Vec::from_raw_parts(ptr, 0, len) });
}

/// Formats the `len` bytes at `ptr` (freeing them) in `style` (0 meta, 1 google, 2 kotlinlang),
/// with `max_width` overriding the style's when non-zero.
///
/// # Safety
/// `ptr` must be a buffer from [`alloc`]`(len)` whose `len` bytes are initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn format(ptr: *mut u8, len: usize, style: u32, max_width: u32) -> *mut u8 {
    let input = unsafe { Vec::from_raw_parts(ptr, len, len) };
    let (status, text) = format_code(&String::from_utf8_lossy(&input), style, max_width);
    let mut result = Vec::with_capacity(5 + text.len());
    result.push(status);
    result.extend_from_slice(&(text.len() as u32).to_le_bytes());
    result.extend_from_slice(text.as_bytes());
    std::mem::ManuallyDrop::new(result.into_boxed_slice()).as_mut_ptr()
}

/// The status and the formatted code or error message. Drops a leading BOM, as ktfmt's CLI does.
pub fn format_code(code: &str, style: u32, max_width: u32) -> (u8, String) {
    let code = code.strip_prefix('\u{feff}').unwrap_or(code);
    let mut options: FormattingOptions = match style {
        0 => META_FORMAT,
        1 => GOOGLE_FORMAT,
        _ => KOTLINLANG_FORMAT,
    };
    if max_width > 0 {
        options.max_width = max_width as i32;
    }
    match ktrs_fmt::format(code, &options) {
        Ok(formatted) => (STATUS_OK, formatted),
        Err(e) => (STATUS_ERROR, e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_through_the_exports() {
        let code = b"fun  f( ) = 1\n";
        let ptr = alloc(code.len());
        let result = unsafe {
            std::ptr::copy_nonoverlapping(code.as_ptr(), ptr, code.len());
            let out = format(ptr, code.len(), 2, 0);
            let len = u32::from_le_bytes(std::slice::from_raw_parts(out.add(1), 4).try_into().unwrap()) as usize;
            let bytes = std::slice::from_raw_parts(out, 5 + len).to_vec();
            dealloc(out, 5 + len);
            bytes
        };
        assert_eq!(result[0], STATUS_OK);
        assert_eq!(&result[5..], b"fun f() = 1\n");
    }

    #[test]
    fn errors_carry_ktfmts_message() {
        let (status, message) = format_code("fun f( {\n", 0, 0);
        assert_eq!(status, STATUS_ERROR);
        assert!(message.starts_with("1:"), "{message}");
    }
}
