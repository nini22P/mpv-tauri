use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, process::Child};

pub struct MpvInstance {
    pub process: Child,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvConfig {
    pub mpv_path: Option<String>,
    pub mpv_args: Option<Vec<String>>,
    pub observed_properties: Option<Vec<String>>,
    pub ipc_timeout_ms: Option<u64>,
    pub show_mpv_output: Option<bool>,
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
