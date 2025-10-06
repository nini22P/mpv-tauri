use log::{error, info, trace, warn};
use raw_window_handle::HasWindowHandle;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::libmpv::{MpvFormat, PropertyValue};
use crate::utils::get_wid;
use crate::Result;
use crate::{libmpv, models::*};

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
        self.init_wid_mode(mpv_config, window_label)?;

        Ok(window_label.to_string())
    }

    fn init_wid_mode(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        let app = self.app.clone();

        let mut initial_options = mpv_config.initial_options.clone();

        if !initial_options.contains_key("wid") {
            let window = self
                .app
                .get_webview_window(window_label)
                .ok_or_else(|| crate::Error::WindowNotFound(window_label.to_string()))?;
            let window_handle = window.window_handle()?;
            let raw_window_handle = window_handle.as_raw();

            let wid = get_wid(raw_window_handle)?;

            initial_options.insert("wid".to_string(), serde_json::json!(wid));
        }

        let Some(mut instances_lock) = self.lock_and_check_existence(window_label)? else {
            return Ok(window_label.to_string());
        };

        let window_label_clone = window_label.to_string();

        let mpv = libmpv::MpvBuilder::new()?
            .set_options(initial_options)?
            .observed_properties(mpv_config.observed_properties)?
            .on_event(move |event| {
                let event_name = format!("mpv-event-{}", window_label_clone);
                if let Err(e) = app.emit_to(&window_label_clone, &event_name, &event) {
                    error!("Failed to emit mpv event to frontend: {}", e);
                }
                Ok(())
            })
            .build()?;

        info!("mpv instance initialized for window '{}'.", window_label);

        let instance = MpvInstance { mpv };
        instances_lock.insert(window_label.to_string(), instance);

        info!("Wid mode initialized for window '{}'.", window_label);

        Ok(window_label.to_string())
    }

    pub fn destroy(&self, window_label: &str) -> Result<()> {
        let instance_to_kill = self.remove_instance(window_label)?;

        if instance_to_kill.is_some() {
            info!(
                "mpv instance for window '{}' has been removed and will be destroyed.",
                window_label,
            );
        } else {
            trace!(
                "No running mpv instance found for window '{}' to destroy.",
                window_label
            );
        }

        Ok(())
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

        self.with_instance(window_label, |instance| {
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

            instance.mpv.command(name, &args_as_slices)?;

            Ok(())
        })
    }

    pub fn set_property(
        &self,
        name: &str,
        value: &serde_json::Value,
        window_label: &str,
    ) -> crate::Result<()> {
        trace!("SET PROPERTY '{}' '{:?}'", name, value);

        self.with_instance(window_label, |instance| {
            let property_value = match value {
                serde_json::Value::Bool(b) => libmpv::PropertyValue::Flag(*b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        libmpv::PropertyValue::Int64(i)
                    } else if let Some(f) = n.as_f64() {
                        libmpv::PropertyValue::Double(f)
                    } else {
                        return Err(crate::Error::SetProperty(format!(
                            "Unsupported number format: {}",
                            n
                        )));
                    }
                }
                serde_json::Value::String(s) => libmpv::PropertyValue::String(s.clone()),
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

            instance.mpv.set_property(name, property_value)?;

            Ok(())
        })
    }

    pub fn get_property(
        &self,
        name: String,
        format: MpvFormat,
        window_label: &str,
    ) -> crate::Result<PropertyValue> {
        self.with_instance(window_label, |instance| {
            let value = match format {
                MpvFormat::String => instance
                    .mpv
                    .get_property_string(&name)
                    .map(PropertyValue::String),
                MpvFormat::Flag => instance
                    .mpv
                    .get_property_flag(&name)
                    .map(PropertyValue::Flag),
                MpvFormat::Int64 => instance
                    .mpv
                    .get_property_int64(&name)
                    .map(PropertyValue::Int64),
                MpvFormat::Double => instance
                    .mpv
                    .get_property_double(&name)
                    .map(PropertyValue::Double),
                MpvFormat::Node => instance
                    .mpv
                    .get_property_node(&name)
                    .map(PropertyValue::Node),
            }?;

            trace!("GET PROPERTY '{}' '{:?}'", name, value);

            Ok(value)
        })
    }

    pub fn set_video_margin_ratio(
        &self,
        ratio: VideoMarginRatio,
        window_label: &str,
    ) -> Result<()> {
        trace!("SET VIDEO MARGIN RATIO '{:?}'", ratio);

        self.with_instance(window_label, |instance| {
            let margins = [
                ("video-margin-ratio-left", ratio.left),
                ("video-margin-ratio-right", ratio.right),
                ("video-margin-ratio-top", ratio.top),
                ("video-margin-ratio-bottom", ratio.bottom),
            ];

            for (property, value_option) in margins {
                if let Some(value) = value_option {
                    let prop_value = libmpv::PropertyValue::Double(value);
                    if let Err(e) = instance.mpv.set_property(property, prop_value) {
                        error!("Failed to set video margin ratio for '{}': {}", property, e);
                    }
                }
            }

            Ok(())
        })
    }

    fn lock_and_check_existence<'a>(
        &'a self,
        window_label: &str,
    ) -> Result<Option<std::sync::MutexGuard<'a, HashMap<String, MpvInstance>>>> {
        let instances_lock = match self.instances.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if instances_lock.contains_key(window_label) {
            info!(
                "mpv instance for window '{}' already exists. Skipping initialization.",
                window_label
            );
            Ok(None)
        } else {
            Ok(Some(instances_lock))
        }
    }

    fn with_instance<F, T>(&self, window_label: &str, operation: F) -> Result<T>
    where
        F: FnOnce(&MpvInstance) -> Result<T>,
    {
        let instances_lock = match self.instances.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering.");
                poisoned.into_inner()
            }
        };

        let instance = instances_lock.get(window_label).ok_or_else(|| {
            crate::Error::InstanceNotFound(format!(
                "mpv instance for window label '{}' not found",
                window_label
            ))
        })?;

        operation(instance)
    }

    fn remove_instance(&self, window_label: &str) -> Result<Option<MpvInstance>> {
        let mut instances_lock = match self.instances.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering.");
                poisoned.into_inner()
            }
        };
        Ok(instances_lock.remove(window_label))
    }
}
