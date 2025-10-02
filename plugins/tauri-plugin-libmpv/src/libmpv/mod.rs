use libmpv_sys;
use std::ffi::{c_void, CString};

mod error;
mod event;
mod models;
mod render;

pub use error::Error;
pub use event::Event;
pub use models::*;
pub use render::{OpenGLInitParams, RenderContext, RenderParam};

pub struct Mpv {
    pub handle: *mut libmpv_sys::mpv_handle,
    wakeup_callback_data: *mut c_void,
}

unsafe impl Send for Mpv {}
unsafe impl Sync for Mpv {}

impl Mpv {
    pub fn builder() -> Result<MpvBuilder, Error> {
        MpvBuilder::new()
    }

    pub fn create_client(&self, name: &str) -> Result<Mpv, Error> {
        let c_name = CString::new(name)?;
        let handle = unsafe { libmpv_sys::mpv_create_client(self.handle, c_name.as_ptr()) };
        if handle.is_null() {
            return Err(Error::ClientCreation);
        }
        Ok(Mpv {
            handle,
            wakeup_callback_data: std::ptr::null_mut(),
        })
    }

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

    pub fn command(&self, name: &str, args: &[&str]) -> Result<(), Error> {
        let c_args: Vec<CString> = std::iter::once(name)
            .chain(args.iter().cloned())
            .map(CString::new)
            .collect::<Result<Vec<_>, _>>()?;

        let mut c_pointers: Vec<*const std::os::raw::c_char> =
            c_args.iter().map(|s| s.as_ptr()).collect();
        c_pointers.push(std::ptr::null());

        let err = unsafe { libmpv_sys::mpv_command(self.handle, c_pointers.as_mut_ptr()) };

        if err < 0 {
            return Err(Error::Command {
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
                libmpv_sys::mpv_format_MPV_FORMAT_STRING as i32,
                &mut data as *mut _ as *mut _,
            )
        };

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
            let rust_string = std::ffi::CStr::from_ptr(data)
                .to_string_lossy()
                .into_owned();
            libmpv_sys::mpv_free(data as *mut _);
            rust_string
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
                libmpv_sys::mpv_format_MPV_FORMAT_FLAG as i32,
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
                libmpv_sys::mpv_format_MPV_FORMAT_INT64 as i32,
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
                libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE as i32,
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
            // libmpv_sys::mpv_format_MPV_FORMAT_NODE => self.get_property_node(key),
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
                        libmpv_sys::mpv_format_MPV_FORMAT_FLAG as i32,
                        &mut val as *mut _ as *mut _,
                    )
                }
                PropertyValue::Int64(mut i) => libmpv_sys::mpv_set_property(
                    self.handle,
                    c_key.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_INT64 as i32,
                    &mut i as *mut _ as *mut _,
                ),
                PropertyValue::Double(mut f) => libmpv_sys::mpv_set_property(
                    self.handle,
                    c_key.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE as i32,
                    &mut f as *mut _ as *mut _,
                ),
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

    pub fn wait_event(&self, timeout: f64) -> Option<Result<Event, String>> {
        let event_ptr = unsafe { libmpv_sys::mpv_wait_event(self.handle, timeout) };
        if event_ptr.is_null() {
            return None;
        }

        let event = unsafe { *event_ptr };
        if event.event_id == libmpv_sys::mpv_event_id_MPV_EVENT_NONE {
            return None;
        }

        match unsafe { Event::from(event) } {
            Ok(Some(event)) => Some(Ok(event)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    pub fn set_wakeup_callback<F>(&mut self, callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        if !self.wakeup_callback_data.is_null() {
            unsafe {
                let _ = Box::from_raw(self.wakeup_callback_data as *mut Box<dyn FnMut() + Send>);
            }
        }

        let boxed_callback: Box<Box<dyn FnMut() + Send + 'static>> = Box::new(Box::new(callback));
        let data_ptr = Box::into_raw(boxed_callback) as *mut c_void;

        self.wakeup_callback_data = data_ptr;

        unsafe extern "C" fn wakeup_trampoline(data: *mut c_void) {
            if data.is_null() {
                return;
            }
            let boxed_callback_ptr = data as *mut Box<dyn FnMut() + Send + 'static>;
            let callback = &mut **boxed_callback_ptr;
            callback();
        }

        unsafe {
            libmpv_sys::mpv_set_wakeup_callback(self.handle, Some(wakeup_trampoline), data_ptr);
        }
    }
}

impl Drop for Mpv {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                libmpv_sys::mpv_set_wakeup_callback(self.handle, None, std::ptr::null_mut());
                libmpv_sys::mpv_terminate_destroy(self.handle);
            }
        }

        if !self.wakeup_callback_data.is_null() {
            unsafe {
                let _ = Box::from_raw(self.wakeup_callback_data as *mut Box<dyn FnMut() + Send>);
            }
        }
    }
}

pub struct MpvBuilder {
    handle: *mut libmpv_sys::mpv_handle,
    built: bool,
}

impl MpvBuilder {
    fn new() -> Result<Self, Error> {
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

fn error_string(err: i32) -> String {
    unsafe {
        let c_str = libmpv_sys::mpv_error_string(err);
        if c_str.is_null() {
            "Unknown error".to_string()
        } else {
            std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned()
        }
    }
}
