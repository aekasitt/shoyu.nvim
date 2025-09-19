/* ~~/src/safe_ffi.rs */

use anyhow::{anyhow, Result};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::panic;
use std::ptr;

use crate::config::RenderConfig;
use crate::renderer::SnippetRenderer;
use crate::syntax;
use crate::themes;

/// Safe wrapper for converting C string to Rust string
fn safe_cstr_to_string(ptr: *const c_char) -> Result<String> {
  if ptr.is_null() {
    return Err(anyhow!("Null pointer provided"));
  }

  let cstr = unsafe { CStr::from_ptr(ptr) };
  cstr
    .to_str()
    .map_err(|e| anyhow!("Invalid UTF-8 in C string: {}", e))
    .map(|s| s.to_owned())
}

/// Safe wrapper for converting Rust string to C string
fn safe_string_to_cstr(s: String) -> *mut c_char {
  match CString::new(s) {
    Ok(cstring) => cstring.into_raw(),
    Err(_) => ptr::null_mut(),
  }
}

/// Safe wrapper for FFI operations with panic catching
fn safe_ffi_operation<F, T>(operation: F) -> *mut c_char
where
  F: FnOnce() -> Result<T> + panic::UnwindSafe,
  T: ToString,
{
  let result = panic::catch_unwind(operation);

  match result {
    Ok(Ok(value)) => safe_string_to_cstr(value.to_string()),
    Ok(Err(_)) => ptr::null_mut(),
    Err(_) => ptr::null_mut(),
  }
}

/// Generate a code snippet image with safe error handling
pub fn safe_generate_snippet_image(
  code: *const c_char,
  language: *const c_char,
  theme: *const c_char,
  config_json: *const c_char,
) -> *mut c_char {
  safe_ffi_operation(|| {
    let code_str = safe_cstr_to_string(code)?;
    let language_str = safe_cstr_to_string(language)?;

    let theme_str = if theme.is_null() {
      "dracula".to_string()
    } else {
      safe_cstr_to_string(theme)?
    };

    let config = if config_json.is_null() {
      RenderConfig::default()
    } else {
      let config_str = safe_cstr_to_string(config_json)?;
      serde_json::from_str(&config_str).map_err(|e| anyhow!("Invalid JSON config: {}", e))?
    };

    let renderer = SnippetRenderer::new(&theme_str, config)?;
    let image_data = renderer.render_snippet(&code_str, &language_str)?;

    Ok(image_data)
  })
}

/// Get available themes with safe error handling
pub fn safe_get_available_themes() -> *mut c_char {
  safe_ffi_operation(|| {
    let themes = themes::get_theme_names();
    let themes_json =
      serde_json::to_string(&themes).map_err(|e| anyhow!("Failed to serialize themes: {}", e))?;
    Ok(themes_json)
  })
}

/// Check if language is supported with safe error handling
pub fn safe_is_language_supported(language: *const c_char) -> c_int {
  let result = panic::catch_unwind(|| -> Result<bool> {
    let language_str = safe_cstr_to_string(language)?;
    Ok(syntax::is_language_supported(&language_str))
  });

  match result {
    Ok(Ok(supported)) => {
      if supported {
        1
      } else {
        0
      }
    }
    Ok(Err(_)) => 0,
    Err(_) => 0,
  }
}

/// Safe memory deallocation
pub fn safe_free_string(s: *mut c_char) {
  if s.is_null() {
    return;
  }

  let _ = panic::catch_unwind(|| unsafe {
    let _ = CString::from_raw(s);
  });
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::ffi::CString;

  #[test]
  fn test_safe_cstr_conversion() {
    let test_str = CString::new("test").unwrap();
    let result = safe_cstr_to_string(test_str.as_ptr());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test");
  }

  #[test]
  fn test_null_pointer_handling() {
    let result = safe_cstr_to_string(ptr::null());
    assert!(result.is_err());
  }

  #[test]
  fn test_safe_string_to_cstr() {
    let ptr = safe_string_to_cstr("test".to_string());
    assert!(!ptr.is_null());
    safe_free_string(ptr);
  }

  #[test]
  fn test_newline_handling() {
    use std::ffi::CString;

    // Test code with newlines
    let code_with_newlines = "fn main() {\n    println!(\"Hello, world!\");\n}";
    let code_cstr = CString::new(code_with_newlines).unwrap();
    let language_cstr = CString::new("rust").unwrap();
    let theme_cstr = CString::new("dracula").unwrap();

    // This should not panic and should return a valid image
    let result = safe_generate_snippet_image(
      code_cstr.as_ptr(),
      language_cstr.as_ptr(),
      theme_cstr.as_ptr(),
      ptr::null(),
    );

    // Should return a valid base64 string (not null)
    assert!(!result.is_null());

    // Clean up
    safe_free_string(result);
  }
}
