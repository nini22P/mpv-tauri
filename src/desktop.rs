use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::{
    ipc::{self},
    models::*,
    process,
};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Mpv<R>> {
    println!("[Tauri Plugin MPV] Plugin registered.");
    let mpv = Mpv(app.clone());
    Ok(mpv)
}

pub struct Mpv<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Mpv<R> {
    pub fn initialize_mpv(
        &self,
        observed_properties: Vec<String>,
        window_label: String,
        mpv_config: HashMap<String, Value>,
    ) -> crate::Result<String> {
        let app_handle = self.0.clone();

        if let Some(webview_window) = app_handle.get_webview_window(&window_label) {
            let handle_result = webview_window.window_handle();

            match handle_result {
                Ok(handle_wrapper) => {
                    let raw_handle = handle_wrapper.as_raw();
                    let window_handle = match raw_handle {
                        RawWindowHandle::Win32(handle) => handle.hwnd.get() as i64,
                        RawWindowHandle::Xlib(handle) => handle.window as i64,
                        RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr() as i64,
                        _ => {
                            eprintln!(
                                "[Tauri Plugin MPV][{}] Unsupported window handle type.",
                                window_label
                            );
                            return Err(crate::Error::UnsupportedPlatform);
                        }
                    };

                    process::init_mpv_process(
                        app_handle,
                        window_handle,
                        window_label.clone(),
                        mpv_config,
                        observed_properties,
                    )?;
                }
                Err(e) => {
                    eprintln!(
                        "[Tauri Plugin MPV]{} Failed to get raw window handle: {:?}",
                        window_label, e,
                    );
                    return Err(crate::Error::WindowHandleError);
                }
            }
        } else {
            eprintln!(
                "[Tauri Plugin MPV][{}] Window with label '{}' not found! Make sure your window exists",
                window_label,
                window_label,
            );
            return Err(crate::Error::WindowHandleError);
        }

        Ok(window_label)
    }

    pub fn send_mpv_command(
        &self,
        command_json: String,
        window_label: String,
    ) -> crate::Result<MpvCommandResponse> {
        ipc::send_command(command_json, window_label)
    }

    pub fn set_video_margin_ratio(
        &self,
        ratio: VideoMarginRatio,
        window_label: String,
    ) -> crate::Result<()> {
        if let Some(left) = ratio.left {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-left", {}]}}"#,
                left
            );
            ipc::send_command(command, window_label.clone())?;
        }

        if let Some(right) = ratio.right {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-right", {}]}}"#,
                right
            );
            ipc::send_command(command, window_label.clone())?;
        }

        if let Some(top) = ratio.top {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-top", {}]}}"#,
                top
            );
            ipc::send_command(command, window_label.clone())?;
        }

        if let Some(bottom) = ratio.bottom {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-bottom", {}]}}"#,
                bottom
            );
            ipc::send_command(command, window_label)?;
        }

        Ok(())
    }

    pub fn destroy_mpv(&self, window_label: String) -> crate::Result<()> {
        process::kill_mpv_process(window_label)
    }
}
