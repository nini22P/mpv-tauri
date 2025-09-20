use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value};
use std::collections::HashMap;

use libmpv2::events::{Event, PropertyData};
use libmpv2::{mpv_end_file_reason, mpv_format, Format, GetData};

use libmpv2_sys::{mpv_node, mpv_node_list};

pub struct MpvInstance {
    pub mpv: libmpv2::Mpv,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvConfig {
    pub initial_properties: Option<HashMap<String, Value>>,
    pub observed_properties: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMarginRatio {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}

pub fn get_format_for_property(property: &str) -> Format {
    match property {
        "pause"
        | "mute"
        | "fullscreen"
        | "loop"
        | "eof-reached"
        | "display-sync-active"
        | "edition"
        | "idle-active"
        | "core-idle"
        | "mixer-active"
        | "playback-abort"
        | "vo-configured"
        | "focused"
        | "deinterlace-active"
        | "demuxer-cache-idle"
        | "demuxer-via-network"
        | "paused-for-cache"
        | "seeking"
        | "seekable"
        | "partially-seekable"
        | "ao-mute" => Format::Flag,
        "playlist-pos"
        | "playlist-pos-1"
        | "playlist-current-pos"
        | "playlist-playing-pos"
        | "playlist-count"
        | "pid"
        | "remaining-file-loops"
        | "remaining-ab-loops"
        | "osd-width"
        | "osd-height"
        | "cursor-autohide"
        | "dwidth"
        | "dheight"
        | "display-width"
        | "display-height"
        | "window-id"
        | "file-size"
        | "estimated-frame-count"
        | "stream-pos"
        | "stream-end"
        | "decoder-frame-drop-count"
        | "frame-drop-count"
        | "vo-delayed-frame-count"
        | "chapters"
        | "cache-speed"
        | "cache-buffering-state"
        | "estimated-frame-number"
        | "width"
        | "height"
        | "audio-bitrate"
        | "video-bitrate"
        | "current-edition"
        | "editions"
        | "chapter"
        | "sub-bitrate"
        | "mistimed-frame-count" => Format::Int64,
        "volume"
        | "speed"
        | "percent-pos"
        | "duration"
        | "time-pos"
        | "time-start"
        | "time-remaining"
        | "playtime-remaining"
        | "audio-speed-correction"
        | "video-speed-correction"
        | "osd-par"
        | "display-hidpi-scale"
        | "display-fps"
        | "avsync"
        | "total-avsync-change"
        | "playback-time"
        | "demuxer-cache-duration"
        | "demuxer-cache-time"
        | "demuxer-start-time"
        | "container-fps"
        | "
  estimated-vf-fps"
        | "audio-pts"
        | "ao-volume"
        | "sub-start"
        | "sub-end"
        | "secondary-sub-start"
        | "secondary-sub-end"
        | "vsync-ratio"
        | "current-window-scale"
        | "estimated-display-fps"
        | "vsync-jitter"
        | "display-swapchain" => Format::Double,
        "filename"
        | "path"
        | "stream-open-filename"
        | "media-title"
        | "file-format"
        | "current-demuxer"
        | "stream-path"
        | "profile-name"
        | "hwdec"
        | "audio-device"
        | "working-directory"
        | "protocol-list"
        | "demuxer-lavf-list"
        | "input-key-list"
        | "mpv-version"
        | "mpv-configuration"
        | "ffmpeg-version"
        | "libass-version"
        | "property-list"
        | "platform"
        | "clock"
        | "display-names"
        | "current-vo"
        | "current-gpu-context"
        | "current-clipboard-backend"
        | "colormatrix"
        | "colormatrix-input-range"
        | "colormatrix-primaries"
        | "current-ao"
        | "hwdec-current"
        | "hwdec-interop"
        | "sub-ass-extradata"
        | "chapter-metadata"
        | "sub-text"
        | "secondary-sub-text"
        | "sub-text-ass"
        | "playlist-path"
        | "current-watch-later-dir" => Format::String,
        _ => Format::String,
    }
}

pub struct MpvJson(Value);

impl MpvJson {
    pub fn into_inner(self) -> Value {
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

unsafe fn convert_node_to_value(node: *const mpv_node) -> crate::Result<Value> {
    Ok(match (*node).format {
        mpv_format::String | mpv_format::OsdString => {
            let s = cstr_to_str((*node).u.string)?;
            Value::String(s.to_string())
        }
        mpv_format::Flag => Value::Bool((*node).u.flag != 0),
        mpv_format::Int64 => Value::Number((*node).u.int64.into()),
        mpv_format::Double => {
            let f = (*node).u.double_;
            serde_json::Number::from_f64(f).map_or(Value::Null, Value::Number)
        }
        mpv_format::Array => {
            let list = (*node).u.list as *const mpv_node_list;
            let mut arr = Vec::with_capacity((*list).num as usize);
            for i in 0..(*list).num {
                arr.push(convert_node_to_value((*list).values.add(i as usize))?);
            }
            Value::Array(arr)
        }
        mpv_format::Map => {
            let list = (*node).u.list as *const mpv_node_list;

            let mut map = JsonMap::new();
            for i in 0..(*list).num {
                let key = cstr_to_str(*(*list).keys.add(i as usize))?;
                let value = convert_node_to_value((*list).values.add(i as usize))?;
                map.insert(key.to_string(), value);
            }
            Value::Object(map)
        }
        _ => Value::Null,
    })
}

unsafe impl GetData for MpvJson {
    fn get_from_c_void<T, F: FnMut(*mut std::ffi::c_void) -> libmpv2::Result<T>>(
        mut fun: F,
    ) -> libmpv2::Result<Self> {
        let mut node = std::mem::MaybeUninit::<mpv_node>::uninit();
        fun(node.as_mut_ptr() as *mut _)?;

        let node_ptr = node.as_mut_ptr();

        let result = match std::panic::catch_unwind(|| unsafe { convert_node_to_value(node_ptr) }) {
            Ok(Ok(value)) => Ok(MpvJson(value)),
            _ => Err(libmpv2::Error::Raw(libmpv2::mpv_error::Generic)),
        };

        unsafe { libmpv2_sys::mpv_free_node_contents(node_ptr) };

        result
    }

    fn get_format() -> Format {
        Format::Node
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
    ParsedJson(Value),
}

impl<'a> From<PropertyData<'a>> for SerializablePropertyData {
    fn from(data: PropertyData<'a>) -> Self {
        match data {
            PropertyData::Str(s) => SerializablePropertyData::Str(s.to_string()),
            PropertyData::OsdStr(s) => SerializablePropertyData::OsdStr(s.to_string()),
            PropertyData::Flag(b) => SerializablePropertyData::Flag(b),
            PropertyData::Int64(i) => SerializablePropertyData::Int64(i),
            PropertyData::Double(d) => SerializablePropertyData::Double(d),
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
    },
    PropertyChange {
        name: String,
        data: SerializablePropertyData,
    },
    EndFile {
        reason: String,
    },
    FileLoaded,
    StartFile,
    VideoReconfig,
    AudioReconfig,
    Seek,
    PlaybackRestart,
    QueueOverflow,
    ClientMessage,
    Deprecated,
    Other(String),
}

impl<'a> From<Event<'a>> for SerializableMpvEvent {
    fn from(event: Event<'a>) -> Self {
        match event {
            Event::PropertyChange { name, change, .. } => {
                let property_name = name.to_string();

                match property_name.as_str() {
                    "playlist"
                    | "metadata"
                    | "filtered-metadata"
                    | "osd-dimensions"
                    | "term-size"
                    | "mouse-pos"
                    | "touch-pos"
                    | "tablet-pos"
                    | "track-list"
                    | "chapter-list"
                    | "perf-info"
                    | "audio-device-list"
                    | "user-data"
                    | "menu-data"
                    | "decoder-list"
                    | "encoder-list"
                    | "profile-list"
                    | "command-list"
                    | "input-bindings"
                    | "video-params"
                    | "video-dec-params"
                    | "video-out-params"
                    | "video-target-params"
                    | "video-frame-info"
                    | "audio-params"
                    | "vo-passes"
                    | "clipboard"
                    | "edition-list"
                    | "demuxer-cache-state"
                    | "audio-out-params"
                    | "current-tracks"
                    | "af"
                    | "vf" => {
                        if let PropertyData::Str(json_string) = change {
                            let parsed_data =
                                serde_json::from_str(json_string).unwrap_or(Value::Null);

                            SerializableMpvEvent::PropertyChange {
                                name: property_name,
                                data: SerializablePropertyData::ParsedJson(parsed_data),
                            }
                        } else {
                            SerializableMpvEvent::PropertyChange {
                                name: property_name,
                                data: change.into(),
                            }
                        }
                    }
                    _ => SerializableMpvEvent::PropertyChange {
                        name: property_name,
                        data: change.into(),
                    },
                }
            }
            Event::Shutdown => SerializableMpvEvent::Shutdown,
            Event::LogMessage {
                prefix,
                level,
                text,
                ..
            } => SerializableMpvEvent::LogMessage {
                prefix: prefix.to_string(),
                level: level.to_string(),
                text: text.to_string(),
            },
            Event::EndFile(reason) => {
                let reason_str = match reason {
                    mpv_end_file_reason::Eof => "eof",
                    mpv_end_file_reason::Stop => "stop",
                    mpv_end_file_reason::Quit => "quit",
                    mpv_end_file_reason::Error => "error",
                    mpv_end_file_reason::Redirect => "redirect",
                    _ => "unknown",
                }
                .to_string();
                SerializableMpvEvent::EndFile { reason: reason_str }
            }
            Event::FileLoaded => SerializableMpvEvent::FileLoaded,
            Event::StartFile => SerializableMpvEvent::StartFile,
            Event::VideoReconfig => SerializableMpvEvent::VideoReconfig,
            Event::AudioReconfig => SerializableMpvEvent::AudioReconfig,
            Event::Seek => SerializableMpvEvent::Seek,
            Event::PlaybackRestart => SerializableMpvEvent::PlaybackRestart,
            Event::QueueOverflow => SerializableMpvEvent::QueueOverflow,
            Event::Deprecated { .. } => SerializableMpvEvent::Deprecated,
            _ => SerializableMpvEvent::Other(format!("{:?}", event)),
        }
    }
}
