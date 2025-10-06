use crate::libmpv::{utils::cstr_to_string, MpvHandle, MpvNode, Result};
use log::warn;
use scopeguard::defer;
use serde::Serialize;
use tauri_plugin_libmpv_sys as libmpv_sys;

pub type EventHandler = Box<dyn FnMut(Event) -> Result<()> + Send + 'static>;

pub struct EventListener {
    pub event_handle: MpvHandle,
}

impl EventListener {
    pub fn wait_event(&self, timeout: f64) -> Option<Result<Event>> {
        let event_ptr = unsafe { libmpv_sys::mpv_wait_event(self.event_handle.inner(), timeout) };

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
}

pub fn start_event_listener(mut event_handler: EventHandler, event_listener: EventListener) {
    std::thread::spawn(move || {
        while let Some(Ok(event)) = event_listener.wait_event(60.0) {
            if let Event::Shutdown = event {
                log::info!("Shutdown event received. Terminating mpv core from event thread.");
                let _ = event_handler(event);
                break;
            }
            if let Err(e) = event_handler(event) {
                log::error!("Error in mpv event handler: {}. Exiting loop.", e);
                break;
            }
        }

        if !event_listener.event_handle.inner().is_null() {
            unsafe { libmpv_sys::mpv_terminate_destroy(event_listener.event_handle.inner()) };
        }
    });
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
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum Event {
    Shutdown,
    LogMessage {
        prefix: String,
        level: String,
        text: String,
    },
    GetPropertyReply {
        name: String,
        data: MpvNode,
        error: i32,
        id: u64,
    },
    SetPropertyReply {
        error: i32,
        id: u64,
    },
    CommandReply {
        result: MpvNode,
        error: i32,
        id: u64,
    },
    StartFile {
        playlist_entry_id: i64,
    },
    EndFile {
        reason: EndFileReason,
        error: i32,
        playlist_entry_id: i64,
        playlist_insert_id: i64,
        playlist_insert_num_entries: i32,
    },
    FileLoaded,
    Idle,
    Tick,
    ClientMessage {
        args: Vec<String>,
    },
    VideoReconfig,
    AudioReconfig,
    Seek,
    PlaybackRestart,
    PropertyChange {
        name: String,
        data: MpvNode,
        id: u64,
    },
    QueueOverflow,
    Hook {
        hook_id: u64,
    },
}

impl Event {
    pub(crate) unsafe fn from(event: libmpv_sys::mpv_event) -> Result<Option<Self>> {
        match event.event_id {
            libmpv_sys::mpv_event_id_MPV_EVENT_SHUTDOWN => Ok(Some(Event::Shutdown)),
            libmpv_sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => {
                let log_msg = &*(event.data as *const libmpv_sys::mpv_event_log_message);

                Ok(Some(Event::LogMessage {
                    prefix: cstr_to_string(log_msg.prefix),
                    level: cstr_to_string(log_msg.level),
                    text: cstr_to_string(log_msg.text),
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
                    id: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_SET_PROPERTY_REPLY => {
                Ok(Some(Event::SetPropertyReply {
                    error: event.error,
                    id: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_COMMAND_REPLY => {
                let cmd = unsafe { *(event.data as *const libmpv_sys::mpv_event_command) };

                Ok(Some(Event::CommandReply {
                    result: MpvNode::from_node(&cmd.result)?,
                    error: event.error,
                    id: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_START_FILE => {
                let start_file =
                    unsafe { *(event.data as *const libmpv_sys::mpv_event_start_file) };

                Ok(Some(Event::StartFile {
                    playlist_entry_id: start_file.playlist_entry_id,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_END_FILE => {
                let end_file = unsafe { *(event.data as *const libmpv_sys::mpv_event_end_file) };

                Ok(Some(Event::EndFile {
                    reason: end_file.reason.into(),
                    playlist_entry_id: end_file.playlist_entry_id,
                    error: end_file.error,
                    playlist_insert_id: end_file.playlist_insert_id,
                    playlist_insert_num_entries: end_file.playlist_insert_num_entries,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_FILE_LOADED => Ok(Some(Event::FileLoaded)),
            libmpv_sys::mpv_event_id_MPV_EVENT_IDLE => Ok(Some(Event::Idle)),
            libmpv_sys::mpv_event_id_MPV_EVENT_TICK => Ok(Some(Event::Tick)),
            libmpv_sys::mpv_event_id_MPV_EVENT_CLIENT_MESSAGE => {
                let client_msg =
                    unsafe { *(event.data as *const libmpv_sys::mpv_event_client_message) };

                let mut args = Vec::new();
                let mut i = 0;

                if !client_msg.args.is_null() {
                    while !(*client_msg.args.add(i)).is_null() {
                        args.push(cstr_to_string(*client_msg.args.add(i)));
                        i += 1;
                    }
                }

                Ok(Some(Event::ClientMessage { args }))
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
                    id: event.reply_userdata,
                }))
            }
            libmpv_sys::mpv_event_id_MPV_EVENT_QUEUE_OVERFLOW => Ok(Some(Event::QueueOverflow)),
            libmpv_sys::mpv_event_id_MPV_EVENT_HOOK => {
                let hook = unsafe { *(event.data as *const libmpv_sys::mpv_event_hook) };

                Ok(Some(Event::Hook { hook_id: hook.id }))
            }
            unknown_id => {
                warn!("Received unknown mpv event ID: {}", unknown_id);
                Ok(None)
            }
        }
    }
}
