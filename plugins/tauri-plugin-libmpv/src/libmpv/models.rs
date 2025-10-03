use base64::{engine::general_purpose, Engine as _};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tauri_plugin_libmpv_sys as libmpv_sys;

use super::utils::cstr_to_string;
use crate::libmpv::Error;

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
            MpvNode::ByteArray(bytes) => Value::String(general_purpose::STANDARD.encode(&bytes)),
        }
    }

    pub(crate) unsafe fn from_node(node: *const libmpv_sys::mpv_node) -> Result<Self, Error> {
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

    pub(crate) unsafe fn from_property(
        property: libmpv_sys::mpv_event_property,
    ) -> Result<Self, Error> {
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
