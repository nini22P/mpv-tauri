use tauri_plugin_libmpv_sys as libmpv_sys;

mod builder;
mod command;
mod error;
mod event;
mod property;
mod utils;

pub use self::{
    builder::MpvBuilder,
    error::{Error, Result},
    event::Event,
    property::{MpvFormat, MpvNode, PropertyValue},
    utils::error_string,
};

#[derive(Clone, Debug)]
pub struct MpvHandle(pub(crate) *mut libmpv_sys::mpv_handle);

impl MpvHandle {
    pub(crate) fn inner(&self) -> *mut libmpv_sys::mpv_handle {
        self.0
    }
}

unsafe impl Send for MpvHandle {}
unsafe impl Sync for MpvHandle {}

#[derive(Clone)]
pub struct Mpv {
    handle: MpvHandle,
}

unsafe impl Send for Mpv {}
unsafe impl Sync for Mpv {}

impl Drop for Mpv {
    fn drop(&mut self) {
        if !self.handle.inner().is_null() {
            unsafe {
                libmpv_sys::mpv_terminate_destroy(self.handle.inner());
            }
            self.handle.0 = std::ptr::null_mut();
        }
    }
}
