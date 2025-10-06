use scopeguard::defer;
use std::ffi::CString;
use tauri_plugin_libmpv_sys as libmpv_sys;

use crate::libmpv::{utils::cstr_to_string, utils::error_string, Error, Mpv, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MpvFormat {
    String,
    Flag,
    Int64,
    Double,
    Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Flag(bool),
    Int64(i64),
    Double(f64),
    Node(MpvNode),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MpvNode {
    None,
    String(String),
    Flag(bool),
    Int64(i64),
    Double(f64),
    NodeArray(Vec<MpvNode>),
    NodeMap(IndexMap<String, MpvNode>),
    ByteArray(Vec<u8>),
}

impl MpvNode {
    pub(crate) unsafe fn from_node(node: *const libmpv_sys::mpv_node) -> Result<Self> {
        match (*node).format {
            libmpv_sys::mpv_format_MPV_FORMAT_NONE => Ok(MpvNode::None),
            libmpv_sys::mpv_format_MPV_FORMAT_STRING => {
                Ok(MpvNode::String(cstr_to_string((*node).u.string)))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_FLAG => Ok(MpvNode::Flag((*node).u.flag != 0)),
            libmpv_sys::mpv_format_MPV_FORMAT_INT64 => Ok(MpvNode::Int64((*node).u.int64)),
            libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE => Ok(MpvNode::Double((*node).u.double_)),
            libmpv_sys::mpv_format_MPV_FORMAT_NODE_ARRAY => {
                let list = &*(*node).u.list;
                let mut vec = Vec::with_capacity(list.num as usize);
                for i in 0..list.num {
                    let child_node = Self::from_node(list.values.add(i as usize))?;
                    vec.push(child_node);
                }
                Ok(MpvNode::NodeArray(vec))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_NODE_MAP => {
                let list = &*(*node).u.list;
                let mut map = IndexMap::with_capacity(list.num as usize);
                for i in 0..list.num {
                    let key = cstr_to_string(*list.keys.add(i as usize));
                    let value_node = Self::from_node(list.values.add(i as usize))?;
                    map.insert(key, value_node);
                }
                Ok(MpvNode::NodeMap(map))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_BYTE_ARRAY => {
                let ba = &*(*node).u.ba;
                let bytes = std::slice::from_raw_parts(ba.data as *const u8, ba.size).to_vec();
                Ok(MpvNode::ByteArray(bytes))
            }
            _ => Err(Error::NodeConversion(format!(
                "Unsupported mpv_node format code: {}",
                (*node).format
            ))),
        }
    }

    pub(crate) unsafe fn from_property(property: libmpv_sys::mpv_event_property) -> Result<Self> {
        match property.format {
            libmpv_sys::mpv_format_MPV_FORMAT_NONE => Ok(MpvNode::None),
            libmpv_sys::mpv_format_MPV_FORMAT_STRING
            | libmpv_sys::mpv_format_MPV_FORMAT_OSD_STRING => {
                let str_ptr = *(property.data as *const *const std::os::raw::c_char);
                Ok(MpvNode::String(cstr_to_string(str_ptr)))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_FLAG => {
                Ok(MpvNode::Flag(*(property.data as *const i32) != 0))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_INT64 => {
                Ok(MpvNode::Int64(*(property.data as *const i64)))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE => {
                Ok(MpvNode::Double(*(property.data as *const f64)))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_NODE => {
                Self::from_node(property.data as *const libmpv_sys::mpv_node)
            }
            _ => Err(Error::PropertyConversion(format!(
                "Unsupported mpv_event_property format code: {}",
                property.format
            ))),
        }
    }
}

impl Mpv {
    pub fn get_property_string(&self, name: &str) -> Result<String> {
        let c_name = CString::new(name)?;

        let mut data: *mut std::os::raw::c_char = std::ptr::null_mut();

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle.inner(),
                c_name.as_ptr(),
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
                name: name.to_string(),
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

    pub fn get_property_flag(&self, name: &str) -> Result<bool> {
        let c_name = CString::new(name)?;

        let mut data: std::os::raw::c_int = 0;

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle.inner(),
                c_name.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
                &mut data as *mut _ as *mut _,
            )
        };

        if err < 0 {
            return Err(Error::GetProperty {
                name: name.to_string(),
                code: error_string(err),
            });
        }

        Ok(data != 0)
    }

    pub fn get_property_int64(&self, name: &str) -> Result<i64> {
        let c_name = CString::new(name)?;

        let mut data: i64 = 0;

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle.inner(),
                c_name.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_INT64,
                &mut data as *mut _ as *mut _,
            )
        };

        if err < 0 {
            return Err(Error::GetProperty {
                name: name.to_string(),
                code: error_string(err),
            });
        }

        Ok(data)
    }

    pub fn get_property_double(&self, name: &str) -> Result<f64> {
        let c_name = CString::new(name)?;

        let mut data: f64 = 0.0;

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle.inner(),
                c_name.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
                &mut data as *mut _ as *mut _,
            )
        };

        if err < 0 {
            return Err(Error::GetProperty {
                name: name.to_string(),
                code: error_string(err),
            });
        }

        Ok(data)
    }

    pub fn get_property_node(&self, name: &str) -> Result<MpvNode> {
        let c_name = CString::new(name)?;

        let mut data: *mut libmpv_sys::mpv_node = std::ptr::null_mut();

        let err = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle.inner(),
                c_name.as_ptr(),
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
                name: name.to_string(),
                code: error_string(err),
            });
        }

        if data.is_null() {
            return Ok(MpvNode::None);
        }

        let node = unsafe { MpvNode::from_node(data)? };

        Ok(node)
    }

    pub fn set_property(&self, name: &str, value: PropertyValue) -> Result<()> {
        let c_name = CString::new(name)?;

        let err = unsafe {
            match value {
                PropertyValue::String(s) => {
                    let c_value = CString::new(s)?;
                    libmpv_sys::mpv_set_property_string(
                        self.handle.inner(),
                        c_name.as_ptr(),
                        c_value.as_ptr(),
                    )
                }
                PropertyValue::Flag(b) => {
                    let mut val: std::os::raw::c_int = if b { 1 } else { 0 };
                    libmpv_sys::mpv_set_property(
                        self.handle.inner(),
                        c_name.as_ptr(),
                        libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
                        &mut val as *mut _ as *mut _,
                    )
                }
                PropertyValue::Int64(mut i) => libmpv_sys::mpv_set_property(
                    self.handle.inner(),
                    c_name.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_INT64,
                    &mut i as *mut _ as *mut _,
                ),
                PropertyValue::Double(mut f) => libmpv_sys::mpv_set_property(
                    self.handle.inner(),
                    c_name.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
                    &mut f as *mut _ as *mut _,
                ),
                PropertyValue::Node(_) => todo!(),
            }
        };

        if err < 0 {
            return Err(Error::SetProperty {
                name: name.to_string(),
                code: error_string(err),
            });
        }

        Ok(())
    }
}
