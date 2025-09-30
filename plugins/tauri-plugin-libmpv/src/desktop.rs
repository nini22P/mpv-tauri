use glutin::context::NotCurrentGlContext;
use glutin::display::{DisplayApiPreference, GlDisplay};
use glutin::surface::{GlSurface, WindowSurface};
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use log::{error, info, trace, warn};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::num::NonZeroU32;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};
use tauri::{Emitter, WebviewWindow};

use crate::error::mpv_error_code_to_name;
use crate::models::*;
use crate::{MpvExt, Result};

trait GlWindow {
    fn build_surface_attributes(
        &self,
        builder: glutin::surface::SurfaceAttributesBuilder<WindowSurface>,
    ) -> std::result::Result<
        glutin::surface::SurfaceAttributes<WindowSurface>,
        raw_window_handle::HandleError,
    >;
}

impl<R: Runtime> GlWindow for WebviewWindow<R> {
    fn build_surface_attributes(
        &self,
        builder: glutin::surface::SurfaceAttributesBuilder<WindowSurface>,
    ) -> std::result::Result<
        glutin::surface::SurfaceAttributes<WindowSurface>,
        raw_window_handle::HandleError,
    > {
        let (w, h) = self
            .inner_size()
            .unwrap()
            .non_zero()
            .expect("invalid zero inner size");
        let handle = self.window_handle()?.as_raw();
        Ok(builder.build(handle, w, h))
    }
}

trait NonZeroU32PhysicalSize {
    fn non_zero(self) -> Option<(NonZeroU32, NonZeroU32)>;
}

impl NonZeroU32PhysicalSize for winit::dpi::PhysicalSize<u32> {
    fn non_zero(self) -> Option<(NonZeroU32, NonZeroU32)> {
        let w = NonZeroU32::new(self.width)?;
        let h = NonZeroU32::new(self.height)?;
        Some((w, h))
    }
}

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

fn get_wid_from_handle(raw_handle: RawWindowHandle) -> Result<i64> {
    match raw_handle {
        RawWindowHandle::Win32(handle) => Ok(handle.hwnd.get() as i64),
        RawWindowHandle::Xlib(handle) => Ok(handle.window as i64),
        RawWindowHandle::AppKit(handle) => Ok(handle.ns_view.as_ptr() as i64),
        _ => Err(crate::Error::UnsupportedPlatform),
    }
}

fn get_proc_address(display: &Arc<glutin::display::Display>, name: &str) -> *mut c_void {
    match CString::new(name) {
        Ok(c_str) => display.get_proc_address(&c_str) as *mut _,
        Err(_) => std::ptr::null_mut(),
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
        displays: Mutex::new(HashMap::new()),
    };
    Ok(mpv)
}

pub struct Mpv<R: Runtime> {
    app: AppHandle<R>,
    pub instances: Mutex<HashMap<String, MpvInstance>>,
    pub displays: Mutex<HashMap<String, Arc<glutin::display::Display>>>,
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
        let mut instances_lock = self.app.mpv().instances.lock().unwrap();

        if instances_lock.contains_key(window_label) {
            info!(
                "mpv instance for window '{}' already exists. Skipping initialization.",
                window_label
            );
            return Ok(window_label.to_string());
        }

        let window = self
            .app
            .get_webview_window(window_label)
            .ok_or_else(|| crate::Error::WindowNotFound(window_label.to_string()))?;
        let window_handle = window.window_handle()?;
        let wid = get_wid_from_handle(window_handle.as_raw())?;

        let mut initial_options = mpv_config.initial_options.clone();
        initial_options.insert("wid".to_string(), serde_json::json!(wid));

        let mpv = Arc::new(Mutex::new(create_mpv_instance(initial_options)?));

        let mpv_clone = mpv.clone();

        let instance = MpvInstance { mpv };
        instances_lock.insert(window_label.to_string(), instance);

        let mpv_client = mpv_clone.lock().unwrap().create_client(None)?;

        info!(
            "Setting up observed properties for window '{}'...",
            window_label
        );

        for (prop, fmt_str) in mpv_config.observed_properties {
            let format = get_format_from_string(&fmt_str)?;
            info!(
                "Observing property '{}' with format '{:?}' for window '{}'",
                prop, format, window_label
            );
            mpv_client.observe_property(&prop, format, 0)?;
        }

        start_event_loop(self.app.clone(), mpv_client, window_label.to_string());

        info!("mpv instance initialized for window '{}'.", window_label);
        Ok(window_label.to_string())
    }

    fn init_render_mode(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        let mut instances_lock = self.app.mpv().instances.lock().unwrap();
        if instances_lock.contains_key(window_label) {
            info!(
                "mpv instance for window '{}' already exists. Skipping initialization.",
                window_label
            );
            return Ok(window_label.to_string());
        }

        let mut initial_options = mpv_config.initial_options.clone();
        initial_options.insert("vo".to_string(), serde_json::json!("libmpv"));

        let mpv = Arc::new(Mutex::new(create_mpv_instance(initial_options)?));

        let mpv_clone = mpv.clone();

        let instance = MpvInstance { mpv };
        instances_lock.insert(window_label.to_string(), instance);

        let mpv_client = mpv_clone.lock().unwrap().create_client(None)?;

        info!(
            "Setting up observed properties for window '{}'...",
            window_label
        );

        for (prop, fmt_str) in mpv_config.observed_properties {
            let format = get_format_from_string(&fmt_str)?;
            info!(
                "Observing property '{}' with format '{:?}' for window '{}'",
                prop, format, window_label
            );
            mpv_client.observe_property(&prop, format, 0)?;
        }

        start_event_loop(self.app.clone(), mpv_client, window_label.to_string());

        let app = self.app.clone();
        let window_label_clone = window_label.to_string();

        thread::spawn(move || {
            if let Err(e) = start_render_thread(app, mpv_clone, &window_label_clone) {
                error!("Render thread exited with error: {}", e);
            }
        });

        Ok(window_label.to_string())
    }

    pub fn destroy(&self, window_label: &str) -> Result<()> {
        {
            let mut displays_lock = match self.app.mpv().displays.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    warn!("Mutex for gl displays was poisoned. Recovering.");
                    poisoned.into_inner()
                }
            };
            displays_lock.remove(window_label);
        }
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
            match instance.mpv.lock().unwrap().command("quit", &[]) {
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

            if let Err(e) = instance.mpv.lock().unwrap().command(name, &args_as_slices) {
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
                serde_json::Value::Bool(b) => instance.mpv.lock().unwrap().set_property(name, *b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        instance.mpv.lock().unwrap().set_property(name, i)
                    } else if let Some(f) = n.as_f64() {
                        instance.mpv.lock().unwrap().set_property(name, f)
                    } else {
                        return Err(crate::Error::SetProperty(format!(
                            "Unsupported number format: {}",
                            n
                        )));
                    }
                }
                serde_json::Value::String(s) => {
                    instance.mpv.lock().unwrap().set_property(name, s.as_str())
                }
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
                            .lock()
                            .unwrap()
                            .get_property::<bool>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::Int64 => instance
                            .mpv
                            .lock()
                            .unwrap()
                            .get_property::<i64>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::Double => instance
                            .mpv
                            .lock()
                            .unwrap()
                            .get_property::<f64>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::String => instance
                            .mpv
                            .lock()
                            .unwrap()
                            .get_property::<String>(&name)
                            .map(serde_json::Value::from),
                        libmpv2::Format::Node => {
                            match instance.mpv.lock().unwrap().get_property::<MpvNode>(&name) {
                                Ok(wrapper) => Ok(wrapper.into_inner()),
                                Err(e) => Err(e),
                            }
                        }
                    }
                } else {
                    instance
                        .mpv
                        .lock()
                        .unwrap()
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
            if let Err(e) = mpv
                .lock()
                .unwrap()
                .set_property("video-margin-ratio-left", ratio.left.unwrap_or(0.0))
            {
                error!("Failed to set video margin ratio: {}", e);
            }
            if let Err(e) = mpv
                .lock()
                .unwrap()
                .set_property("video-margin-ratio-right", ratio.right.unwrap_or(0.0))
            {
                error!("Failed to set video margin ratio: {}", e);
            }
            if let Err(e) = mpv
                .lock()
                .unwrap()
                .set_property("video-margin-ratio-top", ratio.top.unwrap_or(0.0))
            {
                error!("Failed to set video margin ratio: {}", e);
            }
            if let Err(e) = mpv
                .lock()
                .unwrap()
                .set_property("video-margin-ratio-bottom", ratio.bottom.unwrap_or(0.0))
            {
                error!("Failed to set video margin ratio: {}", e);
            }
        }
        Ok(())
    }
}

fn start_render_thread<R: Runtime>(
    app: AppHandle<R>,
    mpv: Arc<Mutex<libmpv2::Mpv>>,
    window_label: &str,
) -> Result<()> {
    let mut displays_lock = app.mpv().displays.lock().unwrap();
    if displays_lock.contains_key(window_label) {
        info!(
            "gl display for window '{}' already exists. Skipping initialization.",
            window_label
        );
        return Ok(());
    }

    let window = app
        .get_webview_window(window_label)
        .ok_or_else(|| crate::Error::WindowNotFound(window_label.to_string()))?;
    let window_handle = window.window_handle()?;
    let raw_window_handle = window_handle.as_raw();
    let display_handle = window.display_handle()?;

    let surface_attributes = window.build_surface_attributes(Default::default()).unwrap();

    let template = glutin::config::ConfigTemplateBuilder::new()
        .compatible_with_native_window(raw_window_handle);

    let display = Arc::new(unsafe {
        let preference = DisplayApiPreference::WglThenEgl(Some(window_handle.as_raw()));
        match glutin::display::Display::new(display_handle.as_raw(), preference) {
            Ok(display) => display,
            Err(e) => {
                error!("Failed to create glutin display: {}", e);
                return Err(crate::Error::UnsupportedPlatform);
            }
        }
    });

    displays_lock.insert(window_label.to_string(), display.clone());

    drop(displays_lock);

    let config = unsafe {
        display
            .find_configs(template.build())
            .unwrap()
            .next()
            .expect("No suitable config found")
    };

    let surface = unsafe {
        display
            .create_window_surface(&config, &surface_attributes)
            .expect("Failed to create window surface")
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

    let mut render_context = match RenderContext::new(
        unsafe { mpv.lock().unwrap().ctx.as_mut() },
        vec![
            RenderParam::ApiType(RenderParamApiType::OpenGl),
            RenderParam::InitParams(OpenGLInitParams {
                get_proc_address,
                ctx: display.clone(),
            }),
        ],
    ) {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("Failed to create render context: {}", e);
            return Err(crate::Error::Initialization(e.to_string()));
        }
    };

    let (event_tx, event_rx) = mpsc::channel::<MpvThreadEvent>();

    let redraw_tx = event_tx.clone();
    let resize_tx = event_tx.clone();

    render_context.set_update_callback(move || {
        redraw_tx.send(MpvThreadEvent::Redraw).ok();
    });

    mpv.lock().unwrap().set_wakeup_callback(move || {
        event_tx.send(MpvThreadEvent::MpvEvents).ok();
    });

    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Resized(_) => {
            resize_tx.send(MpvThreadEvent::Redraw).ok();
        }
        _ => {}
    });

    for event in event_rx {
        match event {
            MpvThreadEvent::Redraw => {
                if let Ok(size) = window.inner_size() {
                    if let Err(e) = render_context.render::<Arc<glutin::display::Display>>(
                        0,
                        size.width as _,
                        size.height as _,
                        true,
                    ) {
                        error!("Failed to render frame: {}", e);
                    }
                }

                surface
                    .swap_buffers(&current_context)
                    .expect("Failed to swap buffers");
            }
            MpvThreadEvent::MpvEvents => {
                while let Some(mpv_event) = mpv.lock().unwrap().wait_event(0.0) {
                    match mpv_event {
                        Ok(libmpv2::events::Event::EndFile(_)) => {
                            trace!("End of file, exiting render thread.");
                            return Ok(());
                        }
                        Ok(libmpv2::events::Event::Shutdown) => {
                            info!("Shutdown event received, exiting render thread.");
                            return Ok(());
                        }
                        Ok(_e) => {
                            // println!("Received mpv event: {:?}", e);
                        }
                        Err(e) => {
                            error!("mpv event error: {}", e);
                            return Err(crate::Error::Mpv(e.to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn create_mpv_instance(
    initial_options: HashMap<String, serde_json::Value>,
) -> Result<libmpv2::Mpv> {
    libmpv2::Mpv::with_initializer(|init| {
        for (key, value) in initial_options {
            match value {
                serde_json::Value::Bool(b) => init.set_option(&key, b)?,
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        init.set_option(&key, i)?
                    } else if let Some(f) = n.as_f64() {
                        init.set_option(&key, f)?
                    }
                }
                serde_json::Value::String(s) => init.set_option(&key, s.as_str())?,
                _ => {}
            }
        }
        Ok(())
    })
    .map_err(|e| crate::Error::Initialization(e.to_string()))
}

fn start_event_loop<R: Runtime>(
    app_handle: AppHandle<R>,
    mut mpv_client: libmpv2::Mpv,
    window_label: String,
) {
    std::thread::spawn(move || 'event_loop: loop {
        let event_result = mpv_client.wait_event(60.0);

        match event_result {
            Some(Ok(event)) => {
                let raw_event_debug = format!("{:?}", event);
                let serializable_event = SerializableMpvEvent::from(event);

                let event_name = format!("mpv-event-{}", window_label);

                if let SerializableMpvEvent::Shutdown = serializable_event {
                    trace!(
                        "mpv event loop for window '{}' finished due to shutdown event.",
                        window_label
                    );
                    let _ = app_handle.emit_to(&window_label, &event_name, &serializable_event);
                    break 'event_loop;
                }

                if let Err(e) = app_handle.emit_to(&window_label, &event_name, &serializable_event)
                {
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
                    window_label, e
                );
                break 'event_loop;
            }
        }
    });
}
