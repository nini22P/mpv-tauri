use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct MpvInstance {
    pub mpv: libmpv2::Mpv,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvConfig {
    pub initial_properties: Option<HashMap<String, serde_json::Value>>,
    pub observed_properties: Option<HashMap<String, String>>,
}

pub struct MpvNode(serde_json::Value);

impl MpvNode {
    pub fn into_inner(self) -> serde_json::Value {
        self.0
    }
}

unsafe fn cstr_to_str<'a>(cstr: *const std::os::raw::c_char) -> crate::Result<&'a str> {
    if cstr.is_null() {
        return Ok("");
    }
    std::ffi::CStr::from_ptr(cstr)
        .to_str()
        .map_err(|e| crate::Error::GetProperty(format!("Invalid UTF-8 sequence: {}", e)))
}

unsafe fn convert_node_to_value(
    node: *const libmpv2_sys::mpv_node,
) -> crate::Result<serde_json::Value> {
    Ok(match (*node).format {
        libmpv2::mpv_format::None => serde_json::Value::Null,
        libmpv2::mpv_format::String | libmpv2::mpv_format::OsdString => {
            let s = cstr_to_str((*node).u.string)?;
            serde_json::Value::String(s.to_string())
        }
        libmpv2::mpv_format::Flag => serde_json::Value::Bool((*node).u.flag != 0),
        libmpv2::mpv_format::Int64 => serde_json::Value::Number((*node).u.int64.into()),
        libmpv2::mpv_format::Double => {
            let f = (*node).u.double_;
            serde_json::Number::from_f64(f)
                .map_or(serde_json::Value::Null, serde_json::Value::Number)
        }
        libmpv2::mpv_format::Array => {
            if (*node).u.list.is_null() {
                return Ok(serde_json::Value::Array(Vec::new()));
            }

            let list = (*node).u.list as *const libmpv2_sys::mpv_node_list;
            let mut arr = Vec::with_capacity((*list).num as usize);
            for i in 0..(*list).num {
                arr.push(convert_node_to_value((*list).values.add(i as usize))?);
            }
            serde_json::Value::Array(arr)
        }
        libmpv2::mpv_format::Map => {
            if (*node).u.list.is_null() {
                return Ok(serde_json::Value::Object(serde_json::Map::new()));
            }

            let list = (*node).u.list as *const libmpv2_sys::mpv_node_list;
            let mut map = serde_json::Map::new();
            for i in 0..(*list).num {
                let key = cstr_to_str(*(*list).keys.add(i as usize))?;
                let value = convert_node_to_value((*list).values.add(i as usize))?;
                map.insert(key.to_string(), value);
            }
            serde_json::Value::Object(map)
        }
        _ => serde_json::Value::Null,
    })
}

unsafe impl libmpv2::GetData for MpvNode {
    fn get_from_c_void<T, F: FnMut(*mut std::ffi::c_void) -> libmpv2::Result<T>>(
        mut fun: F,
    ) -> libmpv2::Result<Self> {
        let mut node = std::mem::MaybeUninit::<libmpv2_sys::mpv_node>::uninit();
        fun(node.as_mut_ptr() as *mut _)?;

        let node_ptr = node.as_mut_ptr();

        let result = match std::panic::catch_unwind(|| unsafe { convert_node_to_value(node_ptr) }) {
            Ok(Ok(value)) => Ok(MpvNode(value)),
            _ => Err(libmpv2::Error::Raw(libmpv2::mpv_error::Generic)),
        };

        unsafe { libmpv2_sys::mpv_free_node_contents(node_ptr) };

        result
    }

    fn get_format() -> libmpv2::Format {
        libmpv2::Format::Node
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SerializablePropertyData {
    Str(String),
    OsdStr(String),
    Flag(bool),
    Int64(i64),
    Double(f64),
}

impl<'a> From<libmpv2::events::PropertyData<'a>> for SerializablePropertyData {
    fn from(data: libmpv2::events::PropertyData<'a>) -> Self {
        match data {
            libmpv2::events::PropertyData::Str(s) => SerializablePropertyData::Str(s.to_string()),
            libmpv2::events::PropertyData::OsdStr(s) => {
                SerializablePropertyData::OsdStr(s.to_string())
            }
            libmpv2::events::PropertyData::Flag(b) => SerializablePropertyData::Flag(b),
            libmpv2::events::PropertyData::Int64(i) => SerializablePropertyData::Int64(i),
            libmpv2::events::PropertyData::Double(d) => SerializablePropertyData::Double(d),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum SerializableMpvEvent {
    Shutdown,
    LogMessage {
        prefix: String,
        level: String,
        text: String,
        log_level: String,
    },
    GetPropertyReply {
        name: String,
        result: SerializablePropertyData,
        reply_userdata: u64,
    },
    SetPropertyReply {
        reply_userdata: u64,
    },
    CommandReply {
        reply_userdata: u64,
    },
    StartFile,
    EndFile {
        reason: String,
    },
    FileLoaded,
    ClientMessage {
        message: Vec<String>,
    },
    VideoReconfig,
    AudioReconfig,
    Seek,
    PlaybackRestart,
    PropertyChange {
        name: String,
        change: SerializablePropertyData,
        reply_userdata: u64,
    },
    QueueOverflow,
    Deprecated,
}

impl<'a> From<libmpv2::events::Event<'a>> for SerializableMpvEvent {
    fn from(event: libmpv2::events::Event<'a>) -> Self {
        match event {
            libmpv2::events::Event::Shutdown => SerializableMpvEvent::Shutdown,
            libmpv2::events::Event::LogMessage {
                prefix,
                level,
                text,
                log_level,
            } => {
                let log_level = match log_level {
                    libmpv2::mpv_log_level::Debug => "debug",
                    libmpv2::mpv_log_level::Error => "error",
                    libmpv2::mpv_log_level::Fatal => "fatal",
                    libmpv2::mpv_log_level::Info => "info",
                    libmpv2::mpv_log_level::None => "none",
                    libmpv2::mpv_log_level::Warn => "warn",
                    libmpv2::mpv_log_level::V => "v",
                    libmpv2::mpv_log_level::Trace => "trace",
                    _ => todo!(),
                }
                .to_string();
                SerializableMpvEvent::LogMessage {
                    prefix: prefix.to_string(),
                    level: level.to_string(),
                    text: text.to_string(),
                    log_level: log_level,
                }
            }
            libmpv2::events::Event::GetPropertyReply {
                name,
                result,
                reply_userdata,
            } => SerializableMpvEvent::GetPropertyReply {
                name: name.to_string(),
                result: result.into(),
                reply_userdata,
            },
            libmpv2::events::Event::SetPropertyReply(reply_userdata) => {
                SerializableMpvEvent::SetPropertyReply { reply_userdata }
            }
            libmpv2::events::Event::CommandReply(reply_userdata) => {
                SerializableMpvEvent::CommandReply { reply_userdata }
            }
            libmpv2::events::Event::StartFile => SerializableMpvEvent::StartFile,
            libmpv2::events::Event::EndFile(reason) => {
                let reason_str = match reason {
                    libmpv2::mpv_end_file_reason::Eof => "eof",
                    libmpv2::mpv_end_file_reason::Stop => "stop",
                    libmpv2::mpv_end_file_reason::Quit => "quit",
                    libmpv2::mpv_end_file_reason::Error => "error",
                    libmpv2::mpv_end_file_reason::Redirect => "redirect",
                    _ => todo!(),
                }
                .to_string();
                SerializableMpvEvent::EndFile { reason: reason_str }
            }
            libmpv2::events::Event::FileLoaded => SerializableMpvEvent::FileLoaded,
            libmpv2::events::Event::ClientMessage(message) => SerializableMpvEvent::ClientMessage {
                message: message.iter().map(|s| s.to_string()).collect(),
            },
            libmpv2::events::Event::VideoReconfig => SerializableMpvEvent::VideoReconfig,
            libmpv2::events::Event::AudioReconfig => SerializableMpvEvent::AudioReconfig,
            libmpv2::events::Event::Seek => SerializableMpvEvent::Seek,
            libmpv2::events::Event::PlaybackRestart => SerializableMpvEvent::PlaybackRestart,
            libmpv2::events::Event::PropertyChange {
                name,
                change,
                reply_userdata,
            } => SerializableMpvEvent::PropertyChange {
                name: name.to_string(),
                change: change.into(),
                reply_userdata,
            },
            libmpv2::events::Event::QueueOverflow => SerializableMpvEvent::QueueOverflow,
            libmpv2::events::Event::Deprecated(_) => SerializableMpvEvent::Deprecated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMarginRatio {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}
