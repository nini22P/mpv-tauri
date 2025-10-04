use log::{error, trace};

pub fn get_wid(raw_window_handle: raw_window_handle::RawWindowHandle) -> crate::Result<i64> {
    match raw_window_handle {
        raw_window_handle::RawWindowHandle::Win32(handle) => {
            trace!("Platform: Windows");
            Ok(handle.hwnd.get() as i64)
        }
        raw_window_handle::RawWindowHandle::Xlib(handle) => {
            trace!("Platform: Linux xlib");
            Ok(handle.window as i64)
        }
        raw_window_handle::RawWindowHandle::Xcb(handle) => {
            trace!("Platform: Linux xcb");
            Ok(handle.window.get() as i64)
        }
        raw_window_handle::RawWindowHandle::AppKit(handle) => {
            trace!("Platform: MacOS");
            Ok(handle.ns_view.as_ptr() as i64)
        }
        raw_window_handle::RawWindowHandle::Wayland(_) => {
            trace!("Platform: Wayland");
            let error_message =
                "Window embedding via --wid is not supported on Wayland.".to_string();
            error!("{}", error_message);
            Err(crate::Error::UnsupportedPlatform(error_message).into())
        }
        _ => {
            trace!("Platform: Unknown");
            let error_message = "Unsupported platform.".to_string();
            error!("{}", error_message);
            Err(crate::Error::UnsupportedPlatform("".to_string()).into())
        }
    }
}

pub fn get_proc_address(
    display: &std::sync::Arc<glutin::display::Display>,
    name: &str,
) -> *mut core::ffi::c_void {
    use glutin::prelude::GlDisplay;
    match std::ffi::CString::new(name) {
        Ok(c_str) => display.get_proc_address(&c_str) as *mut _,
        Err(_) => std::ptr::null_mut(),
    }
}
