use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::{events, ipc, models::*, process};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Mpv<R>> {
    println!("Tauri Plugin MPV: Plugin registered successfully. Call initializeMpv() from frontend when ready.");
    let mpv = Mpv(app.clone());
    Ok(mpv)
}

pub struct Mpv<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Mpv<R> {
    pub fn initialize_mpv(
        &self,
        observed_properties: Option<Vec<String>>,
        window_label: Option<String>,
        mpv_config: Option<HashMap<String, Value>>,
    ) -> crate::Result<String> {
        println!("Tauri Plugin MPV: Starting MPV initialization from frontend...");

        let target_window = window_label.as_deref().unwrap_or("main").to_string();

        let properties = observed_properties.unwrap_or_default();

        println!(
            "Tauri Plugin MPV: Will observe properties: {:?}",
            properties
        );

        let app_handle = self.0.clone();

        println!(
            "Tauri Plugin MPV: Looking for window '{}'...",
            target_window
        );
        if let Some(webview_window) = app_handle.get_webview_window(&target_window) {
            println!(
                "Tauri Plugin MPV: Found window '{}', getting handle...",
                target_window
            );
            let handle_result = webview_window.window_handle();

            match handle_result {
                Ok(handle_wrapper) => {
                    println!(
                        "Tauri Plugin MPV: Got window handle wrapper, extracting raw handle..."
                    );
                    let raw_handle = handle_wrapper.as_raw();
                    let window_handle = match raw_handle {
                        RawWindowHandle::Win32(handle) => handle.hwnd.get() as i64,
                        RawWindowHandle::Xlib(handle) => handle.window as i64,
                        RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr() as i64,
                        _ => {
                            eprintln!(
                                "Tauri Plugin MPV: Unsupported window handle type for mpv --wid"
                            );
                            return Err(crate::Error::UnsupportedPlatform);
                        }
                    };

                    let mpv_config_clone = mpv_config.clone();
                    let target_window_clone_for_process = target_window.clone();

                    process::init_mpv_process(
                        window_handle,
                        &target_window_clone_for_process,
                        mpv_config_clone,
                    )?;

                    let app_handle_clone = app_handle.clone();
                    let target_window_clone = target_window.clone();
                    std::thread::spawn(move || {
                        events::start_event_listener(
                            app_handle_clone,
                            properties,
                            target_window_clone,
                        );
                    });
                }
                Err(e) => {
                    eprintln!("Tauri Plugin MPV: Failed to get raw window handle: {:?}", e);
                    return Err(crate::Error::WindowHandleError);
                }
            }
        } else {
            eprintln!(
                "Tauri Plugin MPV: Window '{}' not found! Make sure your window exists with this label",
                target_window
            );
            return Err(crate::Error::WindowHandleError);
        }

        println!("Tauri Plugin MPV: MPV initialization completed successfully!");
        Ok(target_window)
    }

    pub fn send_mpv_command(
        &self,
        command_json: &str,
        window_label: &str,
    ) -> crate::Result<MpvCommandResponse> {
        ipc::send_command(command_json, window_label)
    }

    pub fn set_video_margin_ratio(
        &self,
        ratio: VideoMarginRatio,
        window_label: &str,
    ) -> crate::Result<()> {
        if let Some(left) = ratio.left {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-left", {}]}}"#,
                left
            );
            ipc::send_command(&command, window_label)?;
        }

        if let Some(right) = ratio.right {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-right", {}]}}"#,
                right
            );
            ipc::send_command(&command, window_label)?;
        }

        if let Some(top) = ratio.top {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-top", {}]}}"#,
                top
            );
            ipc::send_command(&command, window_label)?;
        }

        if let Some(bottom) = ratio.bottom {
            let command = format!(
                r#"{{"command": ["set_property", "video-margin-ratio-bottom", {}]}}"#,
                bottom
            );
            ipc::send_command(&command, window_label)?;
        }

        Ok(())
    }
}
