use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, process::Child};

pub struct MpvInstance {
    pub process: Child,
    pub ipc_timeout: std::time::Duration,
}

fn default_mpv_path() -> String {
    "mpv".to_string()
}

fn default_ipc_timeout() -> u64 {
    2000
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvConfig {
    #[serde(default = "default_mpv_path")]
    pub path: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub observed_properties: Vec<String>,
    #[serde(default = "default_ipc_timeout")]
    pub ipc_timeout_ms: u64,
    #[serde(default)]
    pub show_mpv_output: bool,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MpvCommand {
    pub command: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MpvCommandResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    pub error: String,
    pub request_id: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MpvEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMarginRatio {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}
