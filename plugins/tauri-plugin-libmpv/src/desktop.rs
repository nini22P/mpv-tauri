use std::collections::HashMap;
use std::sync::Mutex;

use log::{error, info, trace, warn};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::de::DeserializeOwned;
use tauri::Emitter;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::error::mpv_error_code_to_name;
use crate::models::*;
use crate::{MpvExt, Result};

fn get_format_from_string(format_str: &str) -> Result<libmpv2::Format> {
    match format_str {
        "string" => Ok(libmpv2::Format::String),
        "flag" => Ok(libmpv2::Format::Flag),
        "int64" => Ok(libmpv2::Format::Int64),
        "double" => Ok(libmpv2::Format::Double),
        "node" => Ok(libmpv2::Format::Node),
        _ => Err(crate::Error::Format(format_str.to_string())),
    }
}

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
    pub fn init(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        let app = self.app.clone();

        if let Some(webview_window) = app.get_webview_window(window_label) {
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

                    let mut instances_lock = app.mpv().instances.lock().unwrap();

                    if instances_lock.contains_key(window_label) {
                        info!(
                            "mpv instance for window '{}' already exists. Skipping initialization.",
                            window_label
                        );
                        return Ok(window_label.to_string());
                    }

                    info!("Initializing mpv instance for window '{}'...", window_label);

                    let mpv = libmpv2::Mpv::with_initializer(|init| {
                        if let Some(options) = mpv_config.initial_options {
                            for (key, value) in options {
                                match value {
                                    serde_json::Value::Bool(b) => init.set_option(&key, b)?,
                                    serde_json::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            init.set_option(&key, i)?
                                        } else if let Some(f) = n.as_f64() {
                                            init.set_option(&key, f)?
                                        }
                                    }
                                    serde_json::Value::String(s) => {
                                        init.set_option(&key, s.as_str())?
                                    }
                                    _ => {}
                                }
                            }
                        }

                        init.set_option("wid", window_handle)?;

                        Ok(())
                    })
                    .map_err(|e| crate::Error::Initialization(e.to_string()))?;

                    let mut mpv_client = mpv
                        .create_client(None)
                        .map_err(|e| crate::Error::ClientCreation(e.to_string()))?;

                    if let Some(observed_properties) = mpv_config.observed_properties {
                        trace!("Observing properties on init: {:?}", observed_properties);
                        for (property, format_str) in observed_properties {
                            let format = get_format_from_string(&format_str)?;
                            trace!("Observing '{}' with format {:?}", property, format);
                            if let Err(e) = mpv_client.observe_property(&property, format, 0) {
                                error!("Failed to observe property '{}': {}", property, e);
                                return Err(crate::Error::Mpv(e.to_string()));
                            }
                        }
                    }

                    info!("mpv instance initialized for window '{}'.", window_label,);

                    let instance = MpvInstance { mpv };
                    instances_lock.insert(window_label.to_string(), instance);

                    let app_handle = app.clone();

                    let window_label_clone = window_label.to_string();

                    std::thread::spawn(move || 'event_loop: loop {
                        let event_result = mpv_client.wait_event(60.0);

                        match event_result {
                            Some(Ok(event)) => {
                                let raw_event_debug = format!("{:?}", event);
                                let serializable_event = SerializableMpvEvent::from(event);

                                let event_name = format!("mpv-event-{}", window_label_clone);

                                if let SerializableMpvEvent::Shutdown = serializable_event {
                                    trace!("mpv event loop for window '{}' finished due to shutdown event.", window_label_clone);
                                    let _ = app_handle.emit_to(
                                        &window_label_clone,
                                        &event_name,
                                        &serializable_event,
                                    );
                                    break 'event_loop;
                                }

                                if let Err(e) = app_handle.emit_to(
                                    &window_label_clone,
                                    &event_name,
                                    &serializable_event,
                                ) {
                                    error!(
                            "Failed to emit mpv event to frontend: {}. Original event: {}",
                            e, raw_event_debug
                        );
                                }
                            }
                            None => continue 'event_loop,
                            Some(Err(e)) => {
                                error!(
                                    "Error in mpv event loop for window '{}': {}. Exiting.",
                                    window_label_clone, e
                                );
                                break 'event_loop;
                            }
                        }
                    });

                    Ok(window_label.to_string())
                }
                Err(e) => {
                    error!(
                        "Failed to get raw window handle for window '{}': {:?}",
                        window_label, e
                    );
                    Err(crate::Error::WindowHandleError)
                }
            }
        } else {
            warn!(
                "Window with label '{}' not found. Please ensure the window exists.",
                window_label
            );
            Err(crate::Error::WindowHandleError)
        }
    }

    pub fn destroy(&self, window_label: &str) -> Result<()> {
        let instance_to_kill = {
            let mut instances_lock = match self.app.mpv().instances.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    warn!("Mutex for mpv instances was poisoned. Recovering.");
                    poisoned.into_inner()
                }
            };
            instances_lock.remove(window_label)
        };

        if let Some(instance) = instance_to_kill {
            match instance.mpv.command("quit", &[]) {
                Ok(_) => {
                    info!(
                        "mpv instance for window '{}' destroyed successfully.",
                        window_label,
                    );
                    Ok(())
                }
                Err(e) => {
                    let error_message = format!(
                        "Failed to destroy mpv instance for window '{}': {}",
                        window_label, e,
                    );
                    error!("{}", error_message);
                    Err(crate::Error::Mpv(error_message))
                }
            }
        } else {
            info!(
            "No running mpv instance found for window '{}' to destroy. It may have already terminated.",
            window_label
        );
            Ok(())
        }
    }

    pub fn command(
        &self,
        name: &str,
        args: &Vec<serde_json::Value>,
        window_label: &str,
    ) -> Result<()> {
        if args.is_empty() {
            trace!("COMMAND '{}'", name);
        } else {
            trace!("COMMAND '{}' '{:?}'", name, args);
        }

        let instances_lock = match self.app.mpv().instances.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering.");
                poisoned.into_inner()
            }
        };

        if let Some(instance) = instances_lock.get(window_label) {
            let string_args: Vec<String> = args
                .iter()
                .map(|v| match v {
                    serde_json::Value::Bool(b) => {
                        if *b {
                            "yes".to_string()
                        } else {
                            "no".to_string()
                        }
                    }
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string().trim_matches('"').to_string(),
                })
                .collect();

            let args_as_slices: Vec<&str> = string_args.iter().map(|s| s.as_str()).collect();

            if let Err(e) = instance.mpv.command(name, &args_as_slices) {
                let error_details = match e {
                    libmpv2::Error::Raw(code) => {
                        format!("{} ({})", mpv_error_code_to_name(code), code)
                    }

                    _ => e.to_string(),
                };

                let error_message = format!(
                    "Failed to execute mpv command '{}' with args '{:?}': {}",
                    name, args, error_details
                );
                error!("{}", error_message);
                return Err(crate::Error::Command(error_message));
            }

            Ok(())
        } else {
            error!("mpv instance for window label '{}' not found", window_label);
            Ok(())
        }
    }

    pub fn set_property(
        &self,
        name: &str,
        value: &serde_json::Value,
        window_label: &str,
    ) -> crate::Result<()> {
        trace!("SET PROPERTY '{}' '{:?}'", name, value);

        let instances_lock = match self.app.mpv().instances.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering.");
                poisoned.into_inner()
            }
        };

        if let Some(instance) = instances_lock.get(window_label) {
            let _ = match value {
                serde_json::Value::Bool(b) => instance.mpv.set_property(name, *b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        instance.mpv.set_property(name, i)
                    } else if let Some(f) = n.as_f64() {
                        instance.mpv.set_property(name, f)
                    } else {
                        return Err(crate::Error::SetProperty(format!(
                            "Unsupported number format: {}",
                            n
                        )));
                    }
                }
                serde_json::Value::String(s) => instance.mpv.set_property(name, s.as_str()),
                serde_json::Value::Null => {
                    return Err(crate::Error::SetProperty(
                        "Cannot set property to null".to_string(),
                    ))
                }
                _ => {
                    return Err(crate::Error::SetProperty(format!(
                        "Unsupported value type for property '{}'",
                        name
                    )))
                }
            };

            Ok(())
        } else {
            Err(crate::Error::SetProperty(format!(
                "mpv instance for window label '{}' not found",
                window_label
            )))
        }
    }

    pub fn get_property(
        &self,
        name: String,
        format_str: Option<String>,
        window_label: &str,
    ) -> crate::Result<serde_json::Value> {
        let instances_lock = match self.app.mpv().instances.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering.");
                poisoned.into_inner()
            }
        };

        if let Some(instance) = instances_lock.get(window_label) {
            let result: std::result::Result<serde_json::Value, libmpv2::Error> = {
                if let Some(s) = format_str {
                    let format = get_format_from_string(&s)?;
                    match format {
                        libmpv2::Format::Flag => instance
                            .mpv
                            .get_property::<bool>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::Int64 => instance
                            .mpv
                            .get_property::<i64>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::Double => instance
                            .mpv
                            .get_property::<f64>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::String => instance
                            .mpv
                            .get_property::<String>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::Node => {
                            match instance.mpv.get_property::<MpvNode>(&name) {
                                Ok(wrapper) => Ok(wrapper.into_inner()),
                                Err(e) => Err(e),
                            }
                        }
                    }
                } else {
                    instance
                        .mpv
                        .get_property::<String>(&name)
                        .map(serde_json::Value::from)
                }
            };

            let value = match result {
                Ok(val) => val,
                Err(e) => return Err(e.into()),
            };

            trace!("GET PROPERTY '{}' '{:?}'", name, value);
            Ok(value)
        } else {
            Err(crate::Error::GetProperty(format!(
                "mpv instance for window label '{}' not found",
                window_label
            )))
        }
    }

    pub fn set_video_margin_ratio(
        &self,
        ratio: VideoMarginRatio,
        window_label: &str,
    ) -> Result<()> {
        trace!("SET VIDEO MARGIN RATIO '{:?}'", ratio);

        let app = self.app.clone();
        let instances_lock = match app.mpv().instances.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering.");
                poisoned.into_inner()
            }
        };

        if let Some(instance) = instances_lock.get(window_label) {
            let mpv = &instance.mpv;
            if let Err(e) = mpv.set_property("video-margin-ratio-left", ratio.left.unwrap_or(0.0)) {
                error!("Failed to set video margin ratio: {}", e);
            }
            if let Err(e) = mpv.set_property("video-margin-ratio-right", ratio.right.unwrap_or(0.0))
            {
                error!("Failed to set video margin ratio: {}", e);
            }
            if let Err(e) = mpv.set_property("video-margin-ratio-top", ratio.top.unwrap_or(0.0)) {
                error!("Failed to set video margin ratio: {}", e);
            }
            if let Err(e) =
                mpv.set_property("video-margin-ratio-bottom", ratio.bottom.unwrap_or(0.0))
            {
                error!("Failed to set video margin ratio: {}", e);
            }
        }
        Ok(())
    }
}
