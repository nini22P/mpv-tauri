use log::error;

pub fn get_wid(raw_window_handle: raw_window_handle::RawWindowHandle) -> crate::Result<i64> {
    match raw_window_handle {
        raw_window_handle::RawWindowHandle::Win32(handle) => Ok(handle.hwnd.get() as i64),
        raw_window_handle::RawWindowHandle::Xlib(handle) => Ok(handle.window as i64),
        raw_window_handle::RawWindowHandle::Xcb(handle) => Ok(handle.window.get() as i64),
        raw_window_handle::RawWindowHandle::AppKit(handle) => Ok(handle.ns_view.as_ptr() as i64),
        raw_window_handle::RawWindowHandle::Wayland(_) => {
            let error_message =
                "Window embedding via --wid is not supported on Wayland.".to_string();
            error!("{}", error_message);
            Err(crate::Error::UnsupportedPlatform(error_message))
        }
        _ => {
            let error_message = "Unsupported platform.".to_string();
            error!("{}", error_message);
            Err(crate::Error::UnsupportedPlatform("".to_string()))
        }
    }
}
