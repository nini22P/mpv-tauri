use tauri::{command, AppHandle, Runtime};

use crate::MpvCommandResponse;
use crate::MpvConfig;
use crate::MpvExt;
use crate::Result;
use crate::VideoMarginRatio;

#[command]
pub(crate) async fn initialize_mpv<R: Runtime>(
    app: AppHandle<R>,
    mpv_config: MpvConfig,
    window_label: &str,
) -> Result<String> {
    app.mpv().initialize_mpv(mpv_config, window_label)
}

#[command]
pub(crate) async fn send_mpv_command<R: Runtime>(
    app: AppHandle<R>,
    command_json: String,
    window_label: String,
) -> Result<MpvCommandResponse> {
    tauri::async_runtime::spawn_blocking(move || {
        app.mpv().send_mpv_command(&command_json, &window_label)
    })
    .await
    .unwrap()
}

#[command]
pub(crate) async fn set_video_margin_ratio<R: Runtime>(
    app: AppHandle<R>,
    ratio: VideoMarginRatio,
    window_label: String,
) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        app.mpv().set_video_margin_ratio(ratio, &window_label)
    })
    .await
    .unwrap()
}

#[command]
pub(crate) async fn destroy_mpv<R: Runtime>(app: AppHandle<R>, window_label: &str) -> Result<()> {
    app.mpv().destroy_mpv(window_label)
}
