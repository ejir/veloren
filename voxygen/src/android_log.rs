//! Minimal logcat reporting for the Android NativeActivity bootstrap.
//!
//! Native stdout/stderr are invisible on Android and the file logger only
//! starts late in [`crate::app::run`], so startup panics would otherwise be
//! silent. This module reports panics and startup-stage breadcrumbs to logcat
//! via `__android_log_write`, installed before anything else runs in
//! `android_main`. No extra dependencies are needed.

use std::{
    ffi::{CString, c_char, c_int},
    panic::PanicHookInfo,
};

const TAG: &str = "veloren";
const INFO: c_int = 4;
const ERROR: c_int = 6;

#[allow(unsafe_code)]
#[link(name = "log")]
unsafe extern "C" {
    fn __android_log_write(priority: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

#[allow(unsafe_code)]
fn write_log(priority: c_int, message: &str) {
    // logcat truncates long lines, so chunk well below the ~4KB limit. Strip
    // NUL bytes so every chunk stays a valid C string.
    let tag = CString::new(TAG).unwrap_or_default();
    for chunk in message.as_bytes().chunks(2000) {
        let text = String::from_utf8_lossy(chunk).replace('\0', "");
        let Ok(text) = CString::new(text) else {
            continue;
        };
        unsafe {
            __android_log_write(priority, tag.as_ptr(), text.as_ptr());
        }
    }
}

/// Report a startup-stage breadcrumb to logcat.
///
/// The last breadcrumb before a crash pinpoints how far bootstrap got, even
/// for failures (such as process exit or signals) that never reach the panic
/// hook.
pub fn stage(message: &str) { write_log(INFO, &format!("[stage] {message}")); }

/// Install a panic hook that reports panics to logcat.
///
/// Must run before anything else in `android_main`. The regular
/// [`crate::panic_handler`] hook installed later in [`crate::app::run`]
/// chains to this one via the default hook, so late panics are reported too.
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info: &PanicHookInfo| {
        let payload = info.payload();
        let reason = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("<non-string panic payload>");
        let location = info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "<unknown location>".to_string());
        // Symbolizing the backtrace can itself fail on stripped binaries;
        // never let crash reporting crash.
        let backtrace = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            format!("{:?}", backtrace::Backtrace::new())
        }))
        .unwrap_or_else(|_| "<backtrace unavailable>".to_string());
        write_log(
            ERROR,
            &format!("VELOREN PANICKED: {reason}\n at {location}\n{backtrace}"),
        );
    }));
}
