use std::collections::HashMap;
use std::sync::Mutex;

use log::{error, info, warn};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::{ipc, models::*, process};
use crate::{MpvInstance, Result};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Mpv<R>> {
    info!("Plugin registered.");
    let mpv = Mpv {
        app: app.clone(),
        instances: Mutex::new(HashMap::new()),
    };
    Ok(mpv)
}

pub struct Mpv<R: Runtime> {
    app: AppHandle<R>,
    pub instances: Mutex<HashMap<String, MpvInstance>>,
}

impl<R: Runtime> Mpv<R> {
    pub fn initialize_mpv(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        let app = self.app.clone();

        if let Some(webview_window) = app.get_webview_window(&window_label) {
            let handle_result = webview_window.window_handle();

            match handle_result {
                Ok(handle_wrapper) => {
                    let raw_handle = handle_wrapper.as_raw();
                    let window_handle = match raw_handle {
                        RawWindowHandle::Win32(handle) => handle.hwnd.get() as i64,
                        RawWindowHandle::Xlib(handle) => handle.window as i64,
                        RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr() as i64,
                        _ => {
                            error!(
                                "Unsupported window handle type for window '{}'.",
                                window_label
                            );
                            return Err(crate::Error::UnsupportedPlatform);
                        }
                    };

                    process::init_mpv_process(&app, window_handle, mpv_config, window_label)?;
                }
                Err(e) => {
                    error!(
                        "Failed to get raw window handle for window '{}': {:?}",
                        window_label, e
                    );
                    return Err(crate::Error::WindowHandleError);
                }
            }
        } else {
            warn!(
                "Window with label '{}' not found. Please ensure the window exists.",
                window_label
            );
            return Err(crate::Error::WindowHandleError);
        }

        Ok(window_label.to_string())
    }

    pub fn send_mpv_command(
        &self,
        mpv_command: &MpvCommand,
        window_label: &str,
    ) -> Result<MpvCommandResponse> {
        ipc::send_command(mpv_command, window_label)
    }

    pub fn set_video_margin_ratio(
        &self,
        ratio: VideoMarginRatio,
        window_label: &str,
    ) -> Result<()> {
        if let Some(left) = ratio.left {
            let mpv_command = MpvCommand {
                command: vec![
                    "set_property".into(),
                    "video-margin-ratio-left".into(),
                    left.into(),
                ],
                request_id: None,
            };
            ipc::send_command(&mpv_command, window_label)?;
        }

        if let Some(right) = ratio.right {
            let mpv_command = MpvCommand {
                command: vec![
                    "set_property".into(),
                    "video-margin-ratio-right".into(),
                    right.into(),
                ],
                request_id: None,
            };
            ipc::send_command(&mpv_command, window_label)?;
        }

        if let Some(top) = ratio.top {
            let mpv_command = MpvCommand {
                command: vec![
                    "set_property".into(),
                    "video-margin-ratio-top".into(),
                    top.into(),
                ],
                request_id: None,
            };
            ipc::send_command(&mpv_command, window_label)?;
        }

        if let Some(bottom) = ratio.bottom {
            let mpv_command = MpvCommand {
                command: vec![
                    "set_property".into(),
                    "video-margin-ratio-bottom".into(),
                    bottom.into(),
                ],
                request_id: None,
            };
            ipc::send_command(&mpv_command, window_label)?;
        }

        Ok(())
    }

    pub fn destroy_mpv(&self, window_label: &str) -> Result<()> {
        process::kill_mpv_process(&self.app, window_label)
    }
}
