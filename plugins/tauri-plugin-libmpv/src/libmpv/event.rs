use crate::libmpv::{utils::cstr_to_string, MpvNode, Result};
use log::warn;
use scopeguard::defer;
use serde::Serialize;
use tauri_plugin_libmpv_sys as libmpv_sys;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MpvLogLevel {
    None,
    Fatal,
    Error,
    Warn,
    Info,
    V,
    Debug,
    Trace,
    Unknown,
}

impl From<libmpv_sys::mpv_log_level> for MpvLogLevel {
    fn from(level: libmpv_sys::mpv_log_level) -> Self {
        match level {
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_NONE => Self::None,
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_FATAL => Self::Fatal,
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_ERROR => Self::Error,
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_WARN => Self::Warn,
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_INFO => Self::Info,
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_V => Self::V,
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_DEBUG => Self::Debug,
            libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_TRACE => Self::Trace,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogMessage {
    prefix: String,
    level: String,
    text: String,
    log_level: MpvLogLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EndFileReason {
    Eof,
    Stop,
    Quit,
    Error,
    Redirect,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StartFile {
    playlist_entry_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EndFile {
    reason: EndFileReason,
    error: i32,
    playlist_entry_id: i64,
    playlist_insert_id: i64,
    playlist_insert_num_entries: i32,
}

impl From<libmpv_sys::mpv_end_file_reason> for EndFileReason {
    fn from(reason: libmpv_sys::mpv_end_file_reason) -> Self {
        match reason {
            libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_EOF => Self::Eof,
            libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_STOP => Self::Stop,
            libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_QUIT => Self::Quit,
            libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_ERROR => Self::Error,
            libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_REDIRECT => Self::Redirect,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hook {
    id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum Event {
    Shutdown,
    LogMessage {
        data: LogMessage,
    },
    GetPropertyReply {
        name: String,
        data: MpvNode,
        error: i32,
        reply_userdata: u64,
    },
    SetPropertyReply {
        error: i32,
        reply_userdata: u64,
    },
    CommandReply {
        data: MpvNode,
        error: i32,
        reply_userdata: u64,
    },
    StartFile {
        data: StartFile,
    },
    EndFile {
        data: EndFile,
    },
    FileLoaded,
    Idle,
    Tick,
    ClientMessage {
        data: Vec<String>,
    },
    VideoReconfig,
    AudioReconfig,
    Seek,
    PlaybackRestart,
    PropertyChange {
        name: String,
        data: MpvNode,
        reply_userdata: u64,
    },
    QueueOverflow,
    Hook {
        data: Hook,
    },
}

impl Event {
    pub(crate) unsafe fn from(event: libmpv_sys::mpv_event) -> Result<Option<Self>> {
        match event.event_id {
            libmpv_sys::mpv_event_id_MPV_EVENT_SHUTDOWN => Ok(Some(Event::Shutdown)),
            libmpv_sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => {
                let log_msg = &*(event.data as *const libmpv_sys::mpv_event_log_message);

                Ok(Some(Event::LogMessage {
                    data: LogMessage {
                        prefix: cstr_to_string(log_msg.prefix),
                        level: cstr_to_string(log_msg.level),
                        text: cstr_to_string(log_msg.text),
                        log_level: log_msg.log_level.into(),
                    },
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY => {
                let property = unsafe { *(event.data as *const libmpv_sys::mpv_event_property) };

                let name = cstr_to_string(property.name);

                let node_ptr = property.data as *const libmpv_sys::mpv_node;

                defer! {
                    unsafe { libmpv_sys::mpv_free_node_contents(node_ptr as *mut _) };
                }

                let node = if node_ptr.is_null() {
                    MpvNode::None
                } else {
                    MpvNode::from_property(property)?
                };

                Ok(Some(Event::GetPropertyReply {
                    name,
                    data: node,
                    error: event.error,
                    reply_userdata: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY => {
                Ok(Some(Event::SetPropertyReply {
                    error: event.error,
                    reply_userdata: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_COMMAND_REPLY => {
                let cmd = unsafe { *(event.data as *const libmpv_sys::mpv_event_command) };

                Ok(Some(Event::CommandReply {
                    data: MpvNode::from_node(&cmd.result)?,
                    error: event.error,
                    reply_userdata: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_START_FILE => {
                let start_file =
                    unsafe { *(event.data as *const libmpv_sys::mpv_event_start_file) };

                Ok(Some(Event::StartFile {
                    data: StartFile {
                        playlist_entry_id: start_file.playlist_entry_id,
                    },
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_END_FILE => {
                let end_file = unsafe { *(event.data as *const libmpv_sys::mpv_event_end_file) };

                Ok(Some(Event::EndFile {
                    data: EndFile {
                        reason: end_file.reason.into(),
                        error: end_file.error,
                        playlist_entry_id: end_file.playlist_entry_id,
                        playlist_insert_id: end_file.playlist_insert_id,
                        playlist_insert_num_entries: end_file.playlist_insert_num_entries,
                    },
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_FILE_LOADED => Ok(Some(Event::FileLoaded)),
            libmpv_sys::mpv_event_id_MPV_EVENT_IDLE => Ok(Some(Event::Idle)),
            libmpv_sys::mpv_event_id_MPV_EVENT_TICK => Ok(Some(Event::Tick)),
            libmpv_sys::mpv_event_id_MPV_EVENT_CLIENT_MESSAGE => {
                let client_msg =
                    unsafe { *(event.data as *const libmpv_sys::mpv_event_client_message) };

                let mut data = Vec::new();
                let mut i = 0;

                if !client_msg.args.is_null() {
                    while !(*client_msg.args.add(i)).is_null() {
                        data.push(cstr_to_string(*client_msg.args.add(i)));
                        i += 1;
                    }
                }

                Ok(Some(Event::ClientMessage { data }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_VIDEO_RECONFIG => Ok(Some(Event::VideoReconfig)),
            libmpv_sys::mpv_event_id_MPV_EVENT_AUDIO_RECONFIG => Ok(Some(Event::AudioReconfig)),
            libmpv_sys::mpv_event_id_MPV_EVENT_SEEK => Ok(Some(Event::Seek)),
            libmpv_sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART => Ok(Some(Event::PlaybackRestart)),
            libmpv_sys::mpv_event_id_MPV_EVENT_PROPERTY_CHANGE => {
                let property = unsafe { *(event.data as *const libmpv_sys::mpv_event_property) };

                let name = cstr_to_string(property.name);

                let node = MpvNode::from_property(property)?;

                Ok(Some(Event::PropertyChange {
                    name,
                    data: node,
                    reply_userdata: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW => Ok(Some(Event::QueueOverflow)),
            libmpv_sys::mpv_event_id_MPV_EVENT_HOOK => {
                let hook = unsafe { *(event.data as *const libmpv_sys::mpv_event_hook) };

                Ok(Some(Event::Hook {
                    data: Hook { id: hook.id },
                }))
            }
            unknown_id => {
                warn!("Received unknown mpv event ID: {}", unknown_id);
                Ok(None)
            }
        }
    }
}
