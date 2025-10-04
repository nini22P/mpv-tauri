use glutin::context::NotCurrentGlContext;
use glutin::display::{Display, DisplayApiPreference, GlDisplay};
use glutin::surface::GlSurface;
use log::{error, info, trace, warn};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use tauri::Emitter;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::libmpv::{MpvFormat, OpenGLInitParams, PropertyValue, RenderParam};
use crate::utils::{get_proc_address, get_wid};
use crate::{libmpv, models::*};
use crate::{MpvExt, Result};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Mpv<R>> {
    info!("Plugin registered.");
    let mpv = Mpv {
        app: app.clone(),
        instances: Mutex::new(HashMap::new()),
        render_contexts: Mutex::new(HashMap::new()),
    };
    Ok(mpv)
}

pub struct Mpv<R: Runtime> {
    app: AppHandle<R>,
    pub instances: Mutex<HashMap<String, MpvInstance>>,
    pub render_contexts: Mutex<HashMap<String, SharedRenderContext>>,
}

impl<R: Runtime> Mpv<R> {
    pub fn init(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        match mpv_config.integration_mode {
            MpvIntegration::Wid => {
                self.init_wid_mode(mpv_config, window_label)?;
            }
            MpvIntegration::Render => {
                self.init_render_mode(mpv_config, window_label)?;
            }
        }

        Ok(window_label.to_string())
    }

    fn init_wid_mode(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        let window = self
            .app
            .get_webview_window(window_label)
            .ok_or_else(|| crate::Error::WindowNotFound(window_label.to_string()))?;
        let window_handle = window.window_handle()?;
        let raw_window_handle = window_handle.as_raw();
        let wid = get_wid(raw_window_handle)?;

        let mut initial_options = mpv_config.initial_options.clone();
        initial_options.insert("wid".to_string(), serde_json::json!(wid));

        let Some(mut instances_lock) = self.lock_and_check_existence(window_label)? else {
            return Ok(window_label.to_string());
        };

        let mpv = create_mpv_instance(initial_options, window_label)?;

        let mpv_client = mpv.create_client("event-client")?;

        let instance = MpvInstance { mpv };
        instances_lock.insert(window_label.to_string(), instance);

        drop(instances_lock);

        start_event_loop(
            self.app.clone(),
            mpv_client,
            mpv_config.observed_properties,
            window_label.to_string(),
        )?;

        info!("Wid mode initialized for window '{}'.", window_label);

        Ok(window_label.to_string())
    }

    fn init_render_mode(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        let mut initial_options = mpv_config.initial_options.clone();
        initial_options.insert("vo".to_string(), serde_json::json!("libmpv"));

        let Some(mut instances_lock) = self.lock_and_check_existence(window_label)? else {
            return Ok(window_label.to_string());
        };

        let mpv = create_mpv_instance(initial_options, window_label)?;

        let mpv_client_for_event = mpv.create_client("event-client")?;
        let mpv_client_for_render = mpv.create_client("render-client")?;

        let instance = MpvInstance { mpv };
        instances_lock.insert(window_label.to_string(), instance);

        drop(instances_lock);

        start_event_loop(
            self.app.clone(),
            mpv_client_for_event,
            mpv_config.observed_properties,
            window_label.to_string(),
        )?;

        let init_rx = spawn_render_thread(
            self.app.clone(),
            mpv_client_for_render,
            window_label.to_string(),
        )?;

        match init_rx.recv() {
            Ok(Ok(())) => {
                info!("Render mode initialized for window '{}'.", window_label);
                Ok(window_label.to_string())
            }
            Ok(Err(e)) => {
                let error_message = format!(
                    "Render thread for window '{}' terminated unexpectedly during setup.",
                    window_label
                );
                error!("{}: {}", error_message, e);
                Err(crate::Error::Initialization(error_message))
            }
            Err(_) => Err(crate::Error::Initialization(format!(
                "Render thread for window '{}' terminated unexpectedly during setup.",
                window_label
            ))),
        }
    }

    pub fn destroy(&self, window_label: &str) -> Result<()> {
        let instance_to_kill = self.remove_instance(window_label)?;

        if let Some(instance) = instance_to_kill {
            instance.mpv.command("quit", &[]).map_err(|e| {
                let error_message = format!(
                    "Failed to destroy mpv instance for window '{}': {}",
                    window_label, e,
                );
                error!("{}", error_message);
                crate::Error::Destroy(error_message)
            })?;

            info!(
                "mpv instance for window '{}' destroyed successfully.",
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
            let value = instance.mpv.get_property(&name, format.into())?;

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

    fn remove_render_context(&self, window_label: &str) -> Result<Option<SharedRenderContext>> {
        let mut render_contexts_lock = match self.render_contexts.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering.");
                poisoned.into_inner()
            }
        };
        Ok(render_contexts_lock.remove(window_label))
    }
}

fn create_mpv_instance(
    initial_options: HashMap<String, serde_json::Value>,
    window_label: &str,
) -> Result<libmpv::Mpv> {
    let mut builder = libmpv::Mpv::builder()?;

    for (key, value) in initial_options {
        let value_str = match value {
            serde_json::Value::Bool(b) => if b { "yes" } else { "no" }.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s,
            _ => continue,
        };
        builder = builder.set_option(&key, &value_str)?;
    }

    let mpv = builder.build()?;

    info!("mpv instance initialized for window '{}'.", window_label);

    Ok(mpv)
}

fn start_event_loop<R: Runtime>(
    app: AppHandle<R>,
    mpv_client: libmpv::Mpv,
    observed_properties: HashMap<String, MpvFormat>,
    window_label: String,
) -> Result<()> {
    info!(
        "Setting up observed properties for window '{}'...",
        window_label
    );

    for (i, (prop, format)) in observed_properties.iter().enumerate() {
        let property_id = (i + 1) as u64;

        info!(
            "Observing property '{}' (ID: {}) with format '{:?}' for window '{}'",
            prop, property_id, format, window_label
        );

        mpv_client.observe_property(prop, (*format).into(), property_id)?;
    }

    std::thread::spawn(move || 'event_loop: loop {
        let event_result = mpv_client.wait_event(60.0);

        match event_result {
            Some(Ok(event)) => {
                let raw_event_debug = format!("{:?}", event);

                let event_name = format!("mpv-event-{}", window_label);

                match event {
                    libmpv::Event::Shutdown => {
                        trace!(
                            "mpv event loop for window '{}' finished due to shutdown event.",
                            window_label
                        );
                        let _ = app.emit_to(&window_label, &event_name, &event);
                        break 'event_loop;
                    }
                    _ => {
                        if let Err(e) = app.emit_to(&window_label, &event_name, &event) {
                            error!(
                                "Failed to emit mpv event to frontend: {}. Original event: {}",
                                e, raw_event_debug
                            );
                        }
                    }
                }
            }
            None => continue 'event_loop,
            Some(Err(e)) => {
                error!(
                    "Error in mpv event loop for window '{}': {}. Exiting.",
                    window_label, e
                );
                break 'event_loop;
            }
        }
    });

    Ok(())
}

fn spawn_render_thread<R: Runtime>(
    app: AppHandle<R>,
    mpv_client: libmpv::Mpv,
    window_label: String,
) -> Result<mpsc::Receiver<Result<()>>> {
    let (init_tx, init_rx) = mpsc::channel::<Result<()>>();

    std::thread::spawn(move || {
        let thread_result = setup_and_run_render_loop(app, mpv_client, &window_label, init_tx);

        if let Err(e) = thread_result {
            error!(
                "Render thread for window '{}' exited with an error: {}",
                window_label, e
            );
        }
    });

    Ok(init_rx)
}

fn setup_and_run_render_loop<R: Runtime>(
    app: AppHandle<R>,
    mut mpv_client: libmpv::Mpv,
    window_label: &str,
    init_tx: mpsc::Sender<Result<()>>,
) -> Result<()> {
    let (event_tx, event_rx) = mpsc::channel::<MpvThreadEvent>();

    let redraw_tx = event_tx.clone();
    let redraw_tx_for_stop = event_tx.clone();
    let resize_tx = event_tx.clone();

    let setup_result = (|| -> Result<Option<(_, _, _, _, _, _)>> {
        let mut render_contexts_lock = match app.mpv().render_contexts.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Mutex for render contexts was poisoned. Recovering.");
                poisoned.into_inner()
            }
        };
        if render_contexts_lock.contains_key(window_label) {
            info!(
                "display for window '{}' already exists. Skipping initialization.",
                window_label
            );
            return Ok(None);
        }

        let window = app
            .get_webview_window(window_label)
            .ok_or_else(|| crate::Error::WindowNotFound(window_label.to_string()))?;
        let window_handle = window.window_handle()?;
        let raw_window_handle = window_handle.as_raw();
        let display_handle = window.display_handle()?;
        let raw_display_handle = display_handle.as_raw();

        let surface_attributes = window.build_surface_attributes(Default::default())?;

        let template = glutin::config::ConfigTemplateBuilder::new()
            .compatible_with_native_window(raw_window_handle);

        let display = Arc::new(unsafe {
            #[cfg(windows)]
            let preference = { DisplayApiPreference::WglThenEgl(Some(raw_window_handle)) };

            #[cfg(all(unix, not(target_os = "macos")))]
            let preference = {
                match raw_window_handle {
                    RawWindowHandle::Wayland(_) => DisplayApiPreference::Egl,
                    RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_) => {
                        DisplayApiPreference::GlxThenEgl(Box::new(
                            winit::platform::x11::register_xlib_error_hook,
                        ))
                    }
                    _ => DisplayApiPreference::Egl,
                }
            };

            #[cfg(target_os = "macos")]
            let preference = DisplayApiPreference::Cgl;

            match Display::new(raw_display_handle, preference) {
                Ok(display) => display,
                Err(e) => {
                    let error_message = format!("Failed to create glutin display: {}", e);
                    error!("{}", error_message);
                    return Err(crate::Error::UnsupportedPlatform(error_message).into());
                }
            }
        });

        let config = unsafe {
            display
                .find_configs(template.build())
                .map_err(|e| {
                    crate::Error::Initialization(format!("Failed to find GL configs: {}", e))
                })?
                .next()
                .ok_or_else(|| {
                    crate::Error::Initialization("No suitable GL config found".to_string())
                })?
        };

        let surface = unsafe {
            display
                .create_window_surface(&config, &surface_attributes)
                .map_err(|e| {
                    crate::Error::Initialization(format!("Failed to create window surface: {}", e))
                })?
        };

        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(Some(raw_window_handle));

        let context = unsafe {
            display
                .create_context(&config, &context_attributes)
                .expect("Failed to create context")
        };

        let current_context = context
            .make_current(&surface)
            .expect("Failed to make context current");

        let render_context = Arc::new(Mutex::new(
            libmpv::RenderContext::new(
                &mpv_client,
                vec![
                    RenderParam::ApiTypeOpenGL,
                    RenderParam::InitParams(OpenGLInitParams {
                        get_proc_address,
                        user_context: display.clone(),
                    }),
                ],
            )
            .map_err(|e| crate::Error::Initialization(e.to_string()))?,
        ));

        render_contexts_lock.insert(window_label.to_string(), render_context.clone());

        drop(render_contexts_lock);

        let render_context_for_callback = render_context.clone();

        let mut render_context_lock = match render_context_for_callback.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Render context mutex was poisoned. Recovering.");
                poisoned.into_inner()
            }
        };

        render_context_lock.set_update_callback(move || {
            redraw_tx.send(MpvThreadEvent::Redraw).ok();
        });

        mpv_client.set_wakeup_callback(move || {
            event_tx.send(MpvThreadEvent::MpvEvents).ok();
        });

        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Resized(_) = event {
                resize_tx.send(MpvThreadEvent::Redraw).ok();
            }
        });

        Ok(Some((
            event_rx,
            window,
            render_context,
            surface,
            current_context,
            display,
        )))
    })();

    let (event_rx, window, render_context, surface, current_context, display) = match setup_result {
        Ok(Some(data)) => data,
        Ok(None) => {
            let _ = init_tx.send(Ok(()));
            return Ok(());
        }
        Err(e) => {
            let _ = init_tx.send(Err(e));
            return Ok(());
        }
    };

    if init_tx.send(Ok(())).is_err() {
        info!(
            "Parent thread disconnected. Aborting render thread for window '{}'.",
            window_label
        );
        return Ok(());
    }

    let mut state = RenderState::Stopped;

    while let Ok(event) = event_rx.recv() {
        match event {
            MpvThreadEvent::Redraw => {
                match &mut state {
                    RenderState::Playing => {}
                    RenderState::Clearing(frames_left) => {
                        *frames_left -= 1;
                        if *frames_left == 0 {
                            state = RenderState::Stopped;
                        } else {
                            std::thread::sleep(std::time::Duration::from_millis(16));
                            redraw_tx_for_stop.send(MpvThreadEvent::Redraw).ok();
                        }
                    }
                    RenderState::Stopped => {}
                }
                let render_context_lock = match render_context.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        warn!("Render context mutex was poisoned. Recovering.");
                        poisoned.into_inner()
                    }
                };

                if let Ok(size) = window.inner_size() {
                    if let Err(e) = render_context_lock.render(size.width as _, size.height as _) {
                        error!("Failed to render frame: {}", e);
                    }
                }

                surface
                    .swap_buffers(&current_context)
                    .expect("Failed to swap buffers");
            }
            MpvThreadEvent::MpvEvents => {
                while let Some(mpv_event) = mpv_client.wait_event(0.0) {
                    match mpv_event {
                        Ok(libmpv::Event::StartFile { .. }) => {
                            state = RenderState::Playing;
                        }
                        Ok(libmpv::Event::EndFile { .. }) => {
                            state = RenderState::Clearing(5);
                            redraw_tx_for_stop.send(MpvThreadEvent::Redraw).ok();
                        }
                        Ok(libmpv::Event::Shutdown) => {
                            info!(
                                "Shutdown event received, exiting render thread for window '{}'.",
                                window_label
                            );

                            drop(current_context);
                            drop(surface);
                            drop(display);

                            app.mpv().remove_render_context(window_label)?;

                            return Ok(());
                        }
                        Ok(_e) => {}
                        Err(e) => {
                            error!("mpv event error: {}", e);

                            app.mpv().remove_render_context(window_label)?;

                            return Err(crate::Error::Render(e.to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
