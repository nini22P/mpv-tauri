use tauri::{command, AppHandle, Runtime};

use crate::MpvCommand;
use crate::MpvCommandResponse;
use crate::MpvConfig;
use crate::MpvExt;
use crate::Result;
use crate::VideoMarginRatio;

#[command]
pub(crate) async fn init<R: Runtime>(
    app: AppHandle<R>,
    mpv_config: MpvConfig,
    window_label: &str,
) -> Result<String> {
    app.mpv().init(mpv_config, window_label)
}

#[command]
pub(crate) async fn command<R: Runtime>(
    app: AppHandle<R>,
    mpv_command: MpvCommand,
    window_label: String,
) -> Result<MpvCommandResponse> {
    tauri::async_runtime::spawn_blocking(move || app.mpv().command(mpv_command, &window_label))
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
pub(crate) async fn destroy<R: Runtime>(app: AppHandle<R>, window_label: &str) -> Result<()> {
    app.mpv().destroy(window_label)
}
