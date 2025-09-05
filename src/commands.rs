use serde_json::Value;
use std::collections::HashMap;
use tauri::{command, AppHandle, Runtime};

use crate::MpvCommandResponse;
use crate::MpvExt;
use crate::Result;
use crate::VideoMarginRatio;

#[command]
pub(crate) async fn initialize_mpv<R: Runtime>(
    app: AppHandle<R>,
    observed_properties: Vec<String>,
    window_label: String,
    mpv_config: HashMap<String, Value>,
) -> Result<String> {
    app.mpv()
        .initialize_mpv(observed_properties, window_label, mpv_config)
}

#[command]
pub(crate) async fn send_mpv_command<R: Runtime>(
    app: AppHandle<R>,
    command_json: String,
    window_label: String,
) -> Result<MpvCommandResponse> {
    app.mpv().send_mpv_command(command_json, window_label)
}

#[command]
pub(crate) async fn set_video_margin_ratio<R: Runtime>(
    app: AppHandle<R>,
    ratio: VideoMarginRatio,
    window_label: String,
) -> Result<()> {
    app.mpv().set_video_margin_ratio(ratio, window_label)
}

#[command]
pub(crate) async fn destroy_mpv<R: Runtime>(app: AppHandle<R>, window_label: String) -> Result<()> {
    app.mpv().destroy_mpv(window_label)
}
