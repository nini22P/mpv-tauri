use libmpv_sys;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MpvFormat {
    String,
    Flag,
    Int64,
    Double,
    Node,
}

impl From<MpvFormat> for libmpv_sys::mpv_format {
    fn from(format: MpvFormat) -> Self {
        match format {
            MpvFormat::String => libmpv_sys::mpv_format_MPV_FORMAT_STRING,
            MpvFormat::Flag => libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
            MpvFormat::Int64 => libmpv_sys::mpv_format_MPV_FORMAT_INT64,
            MpvFormat::Double => libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
            MpvFormat::Node => libmpv_sys::mpv_format_MPV_FORMAT_NODE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Flag(bool),
    Int64(i64),
    Double(f64),
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
    NodeMap(HashMap<String, MpvNode>),
    ByteArray(Vec<u8>),
}

impl MpvNode {
    pub fn into(self) -> serde_json::Value {
        use serde_json::{json, Number, Value};
        match self {
            MpvNode::None => Value::Null,
            MpvNode::String(s) => Value::String(s),
            MpvNode::Flag(b) => Value::Bool(b),
            MpvNode::Int64(i) => Value::Number(Number::from(i)),
            MpvNode::Double(f) => {
                json!(f)
            }
            MpvNode::NodeArray(vec) => {
                let json_vec = vec.into_iter().map(|node| node.into()).collect();
                Value::Array(json_vec)
            }
            MpvNode::NodeMap(map) => {
                let json_map = map.into_iter().map(|(k, v)| (k, v.into())).collect();
                Value::Object(json_map)
            }
            MpvNode::ByteArray(bytes) => Value::String(base64::encode(&bytes)),
        }
    }

    pub(crate) unsafe fn from_property(
        property: libmpv_sys::mpv_event_property,
    ) -> Result<Self, String> {
        let data = match property.format {
            libmpv_sys::mpv_format_MPV_FORMAT_NONE => MpvNode::None,
            libmpv_sys::mpv_format_MPV_FORMAT_STRING
            | libmpv_sys::mpv_format_MPV_FORMAT_OSD_STRING => {
                let str = cstr_to_string(*(property.data as *const *const std::os::raw::c_char));
                MpvNode::String(str)
            }
            libmpv_sys::mpv_format_MPV_FORMAT_FLAG => {
                MpvNode::Flag(*(property.data as *const bool))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_INT64 => {
                MpvNode::Int64(*(property.data as *const i64))
            }
            libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE => {
                MpvNode::Double(*(property.data as *const f64))
            }
            _ => {
                return Err(format!(
                    "Unsupported mpv_node format code: {}",
                    property.format
                ))
            }
        };
        Ok(data)
    }
}

pub unsafe fn cstr_to_string(ptr: *const std::os::raw::c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}
