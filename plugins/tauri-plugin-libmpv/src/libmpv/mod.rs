use libmpv_sys;
use std::ffi::{c_void, CString};

mod error;
mod event;
mod models;
mod render;
mod builder;
mod property;
mod command;
mod utils;

pub use builder::MpvBuilder;
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
