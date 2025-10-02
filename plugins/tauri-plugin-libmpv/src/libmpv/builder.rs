use std::ffi::CString;

use crate::libmpv::{utils::error_string, Error, Mpv};

pub struct MpvBuilder {
    pub(crate) handle: *mut libmpv_sys::mpv_handle,
    pub(crate) built: bool,
}

impl MpvBuilder {
    pub(crate) fn new() -> Result<Self, Error> {
        let handle = unsafe { libmpv_sys::mpv_create() };
        if handle.is_null() {
            return Err(Error::Create);
        }
        Ok(Self {
            handle,
            built: false,
        })
    }

    pub fn set_option(self, key: &str, value: &str) -> Result<Self, Error> {
        let c_key = CString::new(key)?;
        let c_value = CString::new(value)?;

        let err = unsafe {
            libmpv_sys::mpv_set_option_string(self.handle, c_key.as_ptr(), c_value.as_ptr())
        };

        if err < 0 {
            return Err(Error::SetOption {
                key: key.to_string(),
                code: error_string(err),
            });
        }
        Ok(self)
    }

    pub fn set_property(self, key: &str, value: &str) -> Result<Self, Error> {
        let c_key = CString::new(key)?;
        let c_value = CString::new(value)?;

        let err = unsafe {
            libmpv_sys::mpv_set_property_string(self.handle, c_key.as_ptr(), c_value.as_ptr())
        };

        if err < 0 {
            return Err(Error::SetProperty {
                key: key.to_string(),
                code: error_string(err),
            });
        }
        Ok(self)
    }

    pub fn load_config_file(self, path: &str) -> Result<Self, Error> {
        let c_path = CString::new(path)?;
        let err = unsafe { libmpv_sys::mpv_load_config_file(self.handle, c_path.as_ptr()) };

        if err < 0 {
            return Err(Error::LoadConfig {
                path: path.to_string(),
                code: error_string(err),
            });
        }
        Ok(self)
    }

    pub fn build(mut self) -> Result<Mpv, Error> {
        let err = unsafe { libmpv_sys::mpv_initialize(self.handle) };
        if err < 0 {
            return Err(Error::Initialize(error_string(err)));
        }

        self.built = true;
        Ok(Mpv {
            handle: self.handle,
            wakeup_callback_data: std::ptr::null_mut(),
        })
    }
}

impl Drop for MpvBuilder {
    fn drop(&mut self) {
        if !self.built && !self.handle.is_null() {
            unsafe { libmpv_sys::mpv_terminate_destroy(self.handle) };
        }
    }
}
