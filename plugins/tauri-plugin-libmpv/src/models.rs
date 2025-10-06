use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::libmpv;

pub struct MpvInstance {
    pub mpv: crate::libmpv::Mpv,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvConfig {
    #[serde(default)]
    pub initial_options: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub observed_properties: HashMap<String, libmpv::MpvFormat>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMarginRatio {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}
