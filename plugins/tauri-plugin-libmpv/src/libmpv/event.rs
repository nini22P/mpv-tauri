use crate::libmpv::{utils::cstr_to_string, MpvNode};
use libmpv_sys;
use log::warn;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogMessage {
    prefix: String,
    level: String,
    text: String,
    log_level: String,
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
        data: String,
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
    Hook,
}

impl Event {
    pub(crate) unsafe fn from(event: libmpv_sys::mpv_event) -> Result<Option<Self>, String> {
        match event.event_id {
            libmpv_sys::mpv_event_id_MPV_EVENT_SHUTDOWN => Ok(Some(Event::Shutdown)),
            libmpv_sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => {
                let log_msg = &*(event.data as *const libmpv_sys::mpv_event_log_message);
                Ok(Some(Event::LogMessage {
                    data: LogMessage {
                        prefix: cstr_to_string(log_msg.prefix),
                        level: cstr_to_string(log_msg.level),
                        text: cstr_to_string(log_msg.text),
                        log_level: mpv_log_level_to_string(log_msg.log_level),
                    },
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_GET_PROPERTY_REPLY => {
                let property = unsafe { *(event.data as *const libmpv_sys::mpv_event_property) };

                let name = cstr_to_string(property.name);

                let node_ptr = property.data as *const libmpv_sys::mpv_node;
                let node = if node_ptr.is_null() {
                    MpvNode::None
                } else {
                    let parsed_node = MpvNode::from_property(property)?;
                    libmpv_sys::mpv_free_node_contents(node_ptr as *mut _);
                    parsed_node
                };

                Ok(Some(Event::GetPropertyReply {
                    name,
                    data: node,
                    reply_userdata: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY => {
                Ok(Some(Event::SetPropertyReply {
                    reply_userdata: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_COMMAND_REPLY => Ok(Some(Event::CommandReply {
                reply_userdata: event.reply_userdata,
            })),
            libmpv_sys::mpv_event_id_MPV_EVENT_START_FILE => Ok(Some(Event::StartFile)),
            libmpv_sys::mpv_event_id_MPV_EVENT_END_FILE => {
                let end_file = unsafe { &mut *(event.data as *mut libmpv_sys::mpv_event_end_file) };
                Ok(Some(Event::EndFile {
                    data: mpv_end_file_reason_to_string(end_file.reason),
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_FILE_LOADED => Ok(Some(Event::FileLoaded)),
            libmpv_sys::mpv_event_id_MPV_EVENT_IDLE => Ok(Some(Event::Idle)),
            libmpv_sys::mpv_event_id_MPV_EVENT_TICK => Ok(Some(Event::Tick)),
            libmpv_sys::mpv_event_id_MPV_EVENT_CLIENT_MESSAGE => {
                let client_msg =
                    unsafe { &mut *(event.data as *mut libmpv_sys::mpv_event_client_message) };
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
                let property = unsafe { *(event.data as *mut libmpv_sys::mpv_event_property) };

                let name = cstr_to_string(property.name);

                let node = if property.data.is_null() {
                    MpvNode::None
                } else {
                    let parsed_node = match MpvNode::from_property(property) {
                        Ok(node) => node,
                        Err(e) => {
                            return Err(format!("Error parsing property change event: {}", e));
                        }
                    };
                    parsed_node
                };

                Ok(Some(Event::PropertyChange {
                    name,
                    data: node,
                    reply_userdata: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW => Ok(Some(Event::QueueOverflow)),
            libmpv_sys::mpv_event_id_MPV_EVENT_HOOK => Ok(Some(Event::Hook)),
            unknown_id => {
                warn!("Received unknown mpv event ID: {}", unknown_id);
                Ok(None)
            }
        }
    }
}

fn mpv_log_level_to_string(level: libmpv_sys::mpv_log_level) -> String {
    match level {
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_NONE => "none",
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_FATAL => "fatal",
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_ERROR => "error",
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_WARN => "warn",
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_INFO => "info",
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_V => "v",
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_DEBUG => "debug",
        libmpv_sys::mpv_log_level_MPV_LOG_LEVEL_TRACE => "trace",
        _ => "unknown",
    }
    .to_string()
}

fn mpv_end_file_reason_to_string(reason: libmpv_sys::mpv_end_file_reason) -> String {
    match reason {
        libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_EOF => "eof",
        libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_STOP => "stop",
        libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_QUIT => "quit",
        libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_ERROR => "error",
        libmpv_sys::mpv_end_file_reason_MPV_END_FILE_REASON_REDIRECT => "redirect",
        _ => "unknown",
    }
    .to_string()
}
