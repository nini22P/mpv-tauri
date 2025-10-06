use std::ffi::CString;
use tauri_plugin_libmpv_sys as libmpv_sys;

use crate::libmpv::{utils::error_string, Error, Mpv, Result};

impl Mpv {
    pub fn command(&self, name: &str, args: &[&str]) -> Result<()> {
        let c_args: Vec<CString> = std::iter::once(name)
            .chain(args.iter().cloned())
            .map(CString::new)
            .map(|res| res.map_err(|e| Error::InvalidParameter(e.to_string())))
            .collect::<Result<Vec<_>>>()?;

        let mut c_pointers: Vec<*const std::os::raw::c_char> =
            c_args.iter().map(|s| s.as_ptr()).collect();
        c_pointers.push(std::ptr::null());

        let err = unsafe { libmpv_sys::mpv_command(self.handle.0, c_pointers.as_mut_ptr()) };

        if err < 0 {
            return Err(Error::Command {
                name: name.to_string(),
                code: error_string(err),
            });
        }

        Ok(())
    }
}
