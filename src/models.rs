use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub struct MpvEvent {
    /// Event name (as returned by mpv_event_name())
    pub event: String,
    /// The reply_userdata field (opaque user value). Only present if reply_userdata is not 0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// Error string (as returned by mpv_error_string()). Only present if an error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Property name for property-change events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Event data (varies by event type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    // Additional fields for specific event types
    /// Playlist entry ID (for start-file, end-file events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist_entry_id: Option<i64>,
    /// Reason for end-file events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// File error for end-file events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_error: Option<String>,
    /// Playlist insert ID for redirect events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist_insert_id: Option<i64>,
    /// Number of inserted playlist entries for redirect events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist_insert_num_entries: Option<i64>,
    /// Module prefix for log-message events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Log level for log-message events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// Log text for log-message events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Hook ID for hook events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_id: Option<i64>,
    /// Result for command reply events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Arguments for client-message events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

impl MpvEvent {
    /// Create a new MpvEvent with minimal required fields
    pub fn new(event: String) -> Self {
        Self {
            event,
            id: None,
            error: None,
            name: None,
            data: None,
            playlist_entry_id: None,
            reason: None,
            file_error: None,
            playlist_insert_id: None,
            playlist_insert_num_entries: None,
            prefix: None,
            level: None,
            text: None,
            hook_id: None,
            result: None,
            args: None,
        }
    }

    /// Create a property-change event
    pub fn property_change(name: String, data: Option<serde_json::Value>) -> Self {
        Self {
            event: "property-change".to_string(),
            name: Some(name),
            data,
            id: None,
            error: None,
            playlist_entry_id: None,
            reason: None,
            file_error: None,
            playlist_insert_id: None,
            playlist_insert_num_entries: None,
            prefix: None,
            level: None,
            text: None,
            hook_id: None,
            result: None,
            args: None,
        }
    }

    /// Create a start-file event
    pub fn start_file(playlist_entry_id: Option<i64>) -> Self {
        Self {
            event: "start-file".to_string(),
            playlist_entry_id,
            id: None,
            error: None,
            name: None,
            data: None,
            reason: None,
            file_error: None,
            playlist_insert_id: None,
            playlist_insert_num_entries: None,
            prefix: None,
            level: None,
            text: None,
            hook_id: None,
            result: None,
            args: None,
        }
    }

    /// Create an end-file event
    pub fn end_file(
        reason: Option<String>,
        playlist_entry_id: Option<i64>,
        file_error: Option<String>,
    ) -> Self {
        Self {
            event: "end-file".to_string(),
            reason,
            playlist_entry_id,
            file_error,
            id: None,
            error: None,
            name: None,
            data: None,
            playlist_insert_id: None,
            playlist_insert_num_entries: None,
            prefix: None,
            level: None,
            text: None,
            hook_id: None,
            result: None,
            args: None,
        }
    }

    /// Create a log-message event
    pub fn log_message(prefix: String, level: String, text: String) -> Self {
        Self {
            event: "log-message".to_string(),
            prefix: Some(prefix),
            level: Some(level),
            text: Some(text),
            id: None,
            error: None,
            name: None,
            data: None,
            playlist_entry_id: None,
            reason: None,
            file_error: None,
            playlist_insert_id: None,
            playlist_insert_num_entries: None,
            hook_id: None,
            result: None,
            args: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MpvCommand {
    pub command: Vec<serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VideoMarginRatio {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}
