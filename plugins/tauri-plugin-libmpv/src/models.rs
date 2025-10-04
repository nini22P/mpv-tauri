use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::{mpsc, Arc, Mutex},
};
use tauri::{AppHandle, Runtime, WebviewWindow};

use crate::libmpv::{self, RenderContext};

pub struct MpvInstance {
    pub mpv: crate::libmpv::Mpv,
}

pub type GlRenderContext = RenderContext<Arc<glutin::display::Display>>;

pub type SharedRenderContext = Arc<Mutex<GlRenderContext>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MpvIntegration {
    #[default]
    Wid,
    Render,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvConfig {
    #[serde(default)]
    pub integration_mode: MpvIntegration,
    #[serde(default)]
    pub initial_options: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub observed_properties: HashMap<String, libmpv::MpvFormat>,
}

#[derive(Debug)]
pub enum MpvThreadEvent {
    Redraw,
    MpvEvents,
}

#[derive(Debug)]
pub enum RenderState {
    Playing,
    Clearing(u8),
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMarginRatio {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}

pub trait GlWindow {
    fn build_surface_attributes(
        &self,
        builder: glutin::surface::SurfaceAttributesBuilder<glutin::surface::WindowSurface>,
    ) -> std::result::Result<
        glutin::surface::SurfaceAttributes<glutin::surface::WindowSurface>,
        raw_window_handle::HandleError,
    >;
}

impl<R: Runtime> GlWindow for WebviewWindow<R> {
    fn build_surface_attributes(
        &self,
        builder: glutin::surface::SurfaceAttributesBuilder<glutin::surface::WindowSurface>,
    ) -> std::result::Result<
        glutin::surface::SurfaceAttributes<glutin::surface::WindowSurface>,
        raw_window_handle::HandleError,
    > {
        use raw_window_handle::HasWindowHandle;
        let (w, h) = self
            .inner_size()
            .map_err(|_| raw_window_handle::HandleError::Unavailable)?
            .non_zero()
            .ok_or(raw_window_handle::HandleError::Unavailable)?;
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

pub struct MpvRenderResources<R: Runtime> {
    pub window: WebviewWindow<R>,
    pub app: AppHandle<R>,
    pub window_label: String,
    pub render_context: Arc<Mutex<RenderContext<Arc<glutin::display::Display>>>>,
    pub surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    pub current_context: glutin::context::PossiblyCurrentContext,
    pub display: Arc<glutin::display::Display>,
    pub event_rx: mpsc::Receiver<MpvThreadEvent>,
    pub redraw_tx_for_stop: mpsc::Sender<MpvThreadEvent>,
    pub mpv_client: libmpv::Mpv,
}
