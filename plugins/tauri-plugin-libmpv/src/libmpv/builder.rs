use log::info;
use std::{collections::HashMap, ffi::CString, mem::ManuallyDrop, ptr};
use tauri_plugin_libmpv_sys as libmpv_sys;

use crate::libmpv::{
    error_string,
    event::{start_event_listener, EventHandler, EventListener},
    Error, Event, Mpv, MpvFormat, MpvHandle, Result,
};

pub struct MpvBuilder {
    handle: MpvHandle,
    event_handle: MpvHandle,
    event_handler: Option<EventHandler>,
}

impl MpvBuilder {
    pub fn new() -> Result<Self> {
        let handle = unsafe { libmpv_sys::mpv_create() };
        if handle.is_null() {
            return Err(Error::Create);
        }

        let event_handle = unsafe {
            libmpv_sys::mpv_create_client(handle, CString::new("event-client").unwrap().as_ptr())
        };

        if event_handle.is_null() {
            unsafe { libmpv_sys::mpv_terminate_destroy(handle) };
            return Err(Error::ClientCreation);
        }

        Ok(Self {
            handle: MpvHandle(handle),
            event_handle: MpvHandle(event_handle),
            event_handler: None,
        })
    }

    pub fn set_options(self, options: HashMap<String, serde_json::Value>) -> Result<Self> {
        for (name, value) in options {
            let value_str = match value {
                serde_json::Value::Bool(b) => if b { "yes" } else { "no" }.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s,
                _ => continue,
            };

            if name.is_empty() {
                continue;
            }

            let c_name = CString::new(name.clone())?;
            let c_value = CString::new(value_str)?;

            let err = unsafe {
                libmpv_sys::mpv_set_option_string(
                    self.handle.inner(),
                    c_name.as_ptr(),
                    c_value.as_ptr(),
                )
            };

            if err < 0 {
                return Err(Error::SetOption {
                    name,
                    code: error_string(err),
                });
            }
        }
        Ok(self)
    }

    pub fn observed_properties(self, properties: HashMap<String, MpvFormat>) -> Result<Self> {
        for (i, (name, format)) in properties.iter().enumerate() {
            let property_id = (i + 1) as u64;

            info!(
                "Observing property '{}' (ID: {}) with format '{:?}'",
                name, property_id, format
            );

            let c_name = CString::new(name.clone())?;
            let mpv_format = match format {
                MpvFormat::String => libmpv_sys::mpv_format_MPV_FORMAT_STRING,
                MpvFormat::Flag => libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
                MpvFormat::Int64 => libmpv_sys::mpv_format_MPV_FORMAT_INT64,
                MpvFormat::Double => libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
                MpvFormat::Node => libmpv_sys::mpv_format_MPV_FORMAT_NODE,
            };

            let err = unsafe {
                libmpv_sys::mpv_observe_property(
                    self.event_handle.inner(),
                    property_id,
                    c_name.as_ptr(),
                    mpv_format,
                )
            };

            if err < 0 {
                return Err(Error::PropertyObserve {
                    name: name.to_string(),
                    code: error_string(err),
                });
            }
        }
        Ok(self)
    }

    pub fn on_event<F>(mut self, handler: F) -> Self
    where
        F: FnMut(Event) -> Result<()> + Send + 'static,
    {
        self.event_handler = Some(Box::new(handler));
        self
    }

    pub fn build(self) -> Result<Mpv> {
        let err = unsafe { libmpv_sys::mpv_initialize(self.handle.inner()) };
        if err < 0 {
            return Err(Error::Initialize(error_string(err)));
        }

        let builder = ManuallyDrop::new(self);

        let handle = unsafe { ptr::read(&builder.handle) };
        let event_handle = unsafe { ptr::read(&builder.event_handle) };
        let event_handler = unsafe { ptr::read(&builder.event_handler) };

        if event_handler.is_some() {
            let event_listener = EventListener { event_handle };
            start_event_listener(event_handler.unwrap(), event_listener);
        } else if !event_handle.inner().is_null() {
            unsafe { libmpv_sys::mpv_terminate_destroy(event_handle.inner()) };
        }

        let mpv = Mpv { handle };

        Ok(mpv)
    }
}

impl Drop for MpvBuilder {
    fn drop(&mut self) {
        if !self.handle.inner().is_null() {
            unsafe { libmpv_sys::mpv_terminate_destroy(self.handle.inner()) };
        }
    }
}
