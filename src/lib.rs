/* ~~/src/lib.rs */

use std::os::raw::{c_char, c_int};

mod config;
mod font;
mod renderer;
mod safe_ffi;
mod syntax;
mod themes;

/// FFI function to generate a code snippet image
/// Returns a base64-encoded PNG image as a C string
#[no_mangle]
pub extern "C" fn generate_snippet_image(
  code: *const c_char,
  language: *const c_char,
  theme: *const c_char,
  config_json: *const c_char,
) -> *mut c_char {
  safe_ffi::safe_generate_snippet_image(code, language, theme, config_json)
}

/// FFI function to free memory allocated by generate_snippet_image
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
  safe_ffi::safe_free_string(s);
}

/// FFI function to get available themes
#[no_mangle]
pub extern "C" fn get_available_themes() -> *mut c_char {
  safe_ffi::safe_get_available_themes()
}

/// FFI function to validate language support
#[no_mangle]
pub extern "C" fn is_language_supported(language: *const c_char) -> c_int {
  safe_ffi::safe_is_language_supported(language)
}
