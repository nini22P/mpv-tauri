use scopeguard::defer;
use std::ffi::CString;
use tauri_plugin_libmpv_sys as libmpv_sys;

use crate::libmpv::{utils::error_string, Error, Mpv, MpvNode, PropertyValue};

impl Mpv {
    pub fn observe_property(
        &self,
        name: &str,
        format: libmpv_sys::mpv_format,
        userdata: u64,
    ) -> Result<(), Error> {
        let c_name = CString::new(name)?;

        let err = unsafe {
            libmpv_sys::mpv_observe_property(self.handle, userdata, c_name.as_ptr(), format)
        };

        if err < 0 {
            return Err(Error::PropertyObserve {
                name: name.to_string(),
                code: error_string(err),
            });
        }

        Ok(())
    }

    pub fn get_property_string(&self, key: &str) -> Result<String, Error> {
        let c_key = CString::new(key)?;

        let mut data: *mut std::os::raw::c_char = std::ptr::null_mut();

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                c_key.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_STRING,
                &mut data as *mut _ as *mut _,
            )
        };

        defer! {
            if !data.is_null() {
                unsafe { libmpv_sys::mpv_free(data as *mut _) };
            }
        }

        if err < 0 {
            return Err(Error::GetProperty {
                key: key.to_string(),
                code: error_string(err),
            });
        }

        if data.is_null() {
            return Ok("".to_string());
        }

        let result = unsafe {
            std::ffi::CStr::from_ptr(data)
                .to_string_lossy()
                .into_owned()
        };

        Ok(result)
    }

    pub fn get_property_flag(&self, key: &str) -> Result<bool, Error> {
        let c_key = CString::new(key)?;

        let mut data: std::os::raw::c_int = 0;

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                c_key.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
                &mut data as *mut _ as *mut _,
            )
        };

        if err < 0 {
            return Err(Error::GetProperty {
                key: key.to_string(),
                code: error_string(err),
            });
        }

        Ok(data != 0)
    }

    pub fn get_property_int64(&self, key: &str) -> Result<i64, Error> {
        let c_key = CString::new(key)?;

        let mut data: i64 = 0;

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                c_key.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_INT64,
                &mut data as *mut _ as *mut _,
            )
        };

        if err < 0 {
            return Err(Error::GetProperty {
                key: key.to_string(),
                code: error_string(err),
            });
        }

        Ok(data)
    }

    pub fn get_property_double(&self, key: &str) -> Result<f64, Error> {
        let c_key = CString::new(key)?;

        let mut data: f64 = 0.0;

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                c_key.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
                &mut data as *mut _ as *mut _,
            )
        };

        if err < 0 {
            return Err(Error::GetProperty {
                key: key.to_string(),
                code: error_string(err),
            });
        }

        Ok(data)
    }

    pub fn get_property_node(&self, key: &str) -> Result<MpvNode, Error> {
        let c_key = CString::new(key)?;

        let mut data: *mut libmpv_sys::mpv_node = std::ptr::null_mut();

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                c_key.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_NODE,
                &mut data as *mut _ as *mut _,
            )
        };

        defer! {
            if !data.is_null() {
                unsafe { libmpv_sys::mpv_free_node_contents(data) };
            }
        }

        if err < 0 {
            return Err(Error::GetProperty {
                key: key.to_string(),
                code: error_string(err),
            });
        }

        if data.is_null() {
            return Ok(MpvNode::None);
        }

        let node = unsafe { MpvNode::from_node(data)? };

        Ok(node)
    }

    pub fn get_property(
        &self,
        key: &str,
        format: libmpv_sys::mpv_format,
    ) -> Result<PropertyValue, Error> {
        match format {
            libmpv_sys::mpv_format_MPV_FORMAT_STRING => {
                self.get_property_string(key).map(PropertyValue::String)
            }
            libmpv_sys::mpv_format_MPV_FORMAT_FLAG => {
                self.get_property_flag(key).map(PropertyValue::Flag)
            }
            libmpv_sys::mpv_format_MPV_FORMAT_INT64 => {
                self.get_property_int64(key).map(PropertyValue::Int64)
            }
            libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE => {
                self.get_property_double(key).map(PropertyValue::Double)
            }
            libmpv_sys::mpv_format_MPV_FORMAT_NODE => {
                self.get_property_node(key).map(PropertyValue::Node)
            }
            _ => Err(Error::GetProperty {
                key: key.to_string(),
                code: format!("Unsupported format for get_property: {:?}", format),
            }),
        }
    }

    pub fn set_property(&self, key: &str, value: PropertyValue) -> Result<(), Error> {
        let c_key = CString::new(key)?;

        let err = unsafe {
            match value {
                PropertyValue::String(s) => {
                    let c_value = CString::new(s)?;
                    libmpv_sys::mpv_set_property_string(
                        self.handle,
                        c_key.as_ptr(),
                        c_value.as_ptr(),
                    )
                }
                PropertyValue::Flag(b) => {
                    let mut val: std::os::raw::c_int = if b { 1 } else { 0 };
                    libmpv_sys::mpv_set_property(
                        self.handle,
                        c_key.as_ptr(),
                        libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
                        &mut val as *mut _ as *mut _,
                    )
                }
                PropertyValue::Int64(mut i) => libmpv_sys::mpv_set_property(
                    self.handle,
                    c_key.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_INT64,
                    &mut i as *mut _ as *mut _,
                ),
                PropertyValue::Double(mut f) => libmpv_sys::mpv_set_property(
                    self.handle,
                    c_key.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
                    &mut f as *mut _ as *mut _,
                ),
                PropertyValue::Node(_) => todo!(),
            }
        };

        if err < 0 {
            return Err(Error::SetProperty {
                key: key.to_string(),
                code: error_string(err),
            });
        }

        Ok(())
    }
}
