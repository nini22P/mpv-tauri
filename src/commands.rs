use serde_json::Value;
use std::collections::HashMap;
use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::MpvExt;
use crate::Result;

#[command]
pub(crate) async fn initialize_mpv<R: Runtime>(
    app: AppHandle<R>,
    observed_properties: Option<Vec<String>>,
    window_label: Option<String>,
    mpv_config: Option<HashMap<String, Value>>,
) -> Result<String> {
    app.mpv()
        .initialize_mpv(observed_properties, window_label, mpv_config)
}

#[command]
pub(crate) async fn send_mpv_command<R: Runtime>(
    app: AppHandle<R>,
    command_json: String,
    window_label: &str,
) -> Result<String> {
    app.mpv().send_mpv_command(&command_json, window_label)
}

#[command]
pub(crate) async fn set_video_margin_ratio<R: Runtime>(
    app: AppHandle<R>,
    ratio: VideoMarginRatio,
    window_label: &str,
) -> Result<()> {
    app.mpv().set_video_margin_ratio(ratio, window_label)
}
