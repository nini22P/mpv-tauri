use tauri::{command, AppHandle, Runtime};

use crate::libmpv::MpvFormat;
use crate::libmpv::PropertyValue;
use crate::MpvConfig;
use crate::MpvExt;
use crate::Result;
use crate::VideoMarginRatio;

#[command]
pub(crate) async fn init<R: Runtime>(
    app: AppHandle<R>,
    mpv_config: MpvConfig,
    window_label: String,
) -> Result<String> {
    app.mpv().init(mpv_config, &window_label)
}

#[command]
pub(crate) async fn destroy<R: Runtime>(app: AppHandle<R>, window_label: String) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || app.mpv().destroy(&window_label))
        .await
        .map_err(|e| crate::Error::Destroy(e.to_string()))?
        .map_err(Into::into)
}

#[command]
pub(crate) async fn command<R: Runtime>(
    app: AppHandle<R>,
    name: String,
    args: Vec<serde_json::Value>,
    window_label: String,
) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || app.mpv().command(&name, &args, &window_label))
        .await
        .map_err(|e| crate::Error::Command(e.to_string()))?
}

#[command]
pub(crate) async fn set_property<R: Runtime>(
    app: AppHandle<R>,
    name: String,
    value: serde_json::Value,
    window_label: String,
) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        app.mpv().set_property(&name, &value, &window_label)
    })
    .await
    .map_err(|e| crate::Error::SetProperty(e.to_string()))?
}

#[command]
pub(crate) async fn get_property<R: Runtime>(
    app: AppHandle<R>,
    name: String,
    format: MpvFormat,
    window_label: String,
) -> Result<PropertyValue> {
    tauri::async_runtime::spawn_blocking(move || {
        app.mpv().get_property(name, format, &window_label)
    })
    .await
    .map_err(|e| crate::Error::GetProperty(e.to_string()))?
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
    .map_err(|e| crate::Error::Command(e.to_string()))?
}
