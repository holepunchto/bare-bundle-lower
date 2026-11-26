mod bundle;
mod commonjs;
mod lower;

use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::{ptr, slice};

use lower::{Diagnostic, Options};

#[repr(C)]
pub struct bare_bundle_lower_string_t {
    data: *const u8,
    len: usize,
}

#[repr(C)]
pub struct bare_bundle_lower_options_t {
    target: bare_bundle_lower_string_t,
    format: bare_bundle_lower_string_t,
}

#[repr(C)]
pub struct bare_bundle_lower_diagnostic_t {
    url: bare_bundle_lower_string_t,
    start: u32,
    end: u32,
    message: bare_bundle_lower_string_t,
}

#[repr(C)]
pub struct bare_bundle_lower_result_t {
    data: *const u8,
    len: usize,
    diagnostics: *const bare_bundle_lower_diagnostic_t,
    diagnostics_len: usize,
    internal: *mut c_void,
}

struct Owned {
    data: Vec<u8>,
    diagnostics: Vec<Diagnostic>,
    views: Vec<bare_bundle_lower_diagnostic_t>,
}

/// # Safety
///
/// `options` and `result` must be valid pointers, and `input` must point to
/// `len` readable bytes that are not modified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bare_bundle_lower(
    options: *const bare_bundle_lower_options_t,
    input: *const u8,
    len: usize,
    result: *mut bare_bundle_lower_result_t,
) -> i32 {
    let outcome = catch_unwind(AssertUnwindSafe(|| unsafe {
        let options = &*options;

        let target = to_string(&options.target);

        let Some(target) = lower::parse_target(&target) else {
            return Err(vec![Diagnostic::new(
                "",
                format!("Unknown target '{target}'"),
            )]);
        };

        let format = to_string(&options.format);

        let Some(format) = lower::parse_format(&format) else {
            return Err(vec![Diagnostic::new(
                "",
                format!("Unknown format '{format}'"),
            )]);
        };

        lower::lower(slice_or_empty(input, len), &Options { target, format })
    }));

    let (data, diagnostics) = match outcome {
        Ok(Ok(data)) => (data, Vec::new()),
        Ok(Err(diagnostics)) => (Vec::new(), diagnostics),
        Err(_) => (
            Vec::new(),
            vec![Diagnostic::new("", "Internal lowering error")],
        ),
    };

    let views = diagnostics
        .iter()
        .map(|diagnostic| bare_bundle_lower_diagnostic_t {
            url: to_view(&diagnostic.url),
            start: diagnostic.start,
            end: diagnostic.end,
            message: to_view(&diagnostic.message),
        })
        .collect();

    let owned = Box::new(Owned {
        data,
        diagnostics,
        views,
    });

    let status = if owned.diagnostics.is_empty() { 0 } else { -1 };

    unsafe {
        *result = bare_bundle_lower_result_t {
            data: owned.data.as_ptr(),
            len: owned.data.len(),
            diagnostics: owned.views.as_ptr(),
            diagnostics_len: owned.views.len(),
            internal: Box::into_raw(owned).cast(),
        };
    }

    status
}

/// # Safety
///
/// `result` must have been filled in by `bare_bundle_lower()` and not
/// already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bare_bundle_lower_result_free(result: *mut bare_bundle_lower_result_t) {
    unsafe {
        let result = &mut *result;

        if !result.internal.is_null() {
            drop(Box::from_raw(result.internal.cast::<Owned>()));
        }

        result.data = ptr::null();
        result.len = 0;
        result.diagnostics = ptr::null();
        result.diagnostics_len = 0;
        result.internal = ptr::null_mut();
    }
}

unsafe fn slice_or_empty<'a, T>(data: *const T, len: usize) -> &'a [T] {
    if data.is_null() || len == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(data, len) }
    }
}

unsafe fn to_string(string: &bare_bundle_lower_string_t) -> String {
    String::from_utf8_lossy(unsafe { slice_or_empty(string.data, string.len) }).into_owned()
}

fn to_view(string: &str) -> bare_bundle_lower_string_t {
    bare_bundle_lower_string_t {
        data: string.as_ptr(),
        len: string.len(),
    }
}
