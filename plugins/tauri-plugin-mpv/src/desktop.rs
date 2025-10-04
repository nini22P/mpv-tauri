use log::info;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::{ipc, models::*, process, MpvExt};
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
    pub fn init(&self, mpv_config: MpvConfig, window_label: &str) -> Result<String> {
        let app = self.app.clone();

        process::init_mpv_process(&app, mpv_config, window_label)?;

        Ok(window_label.to_string())
    }

    pub fn destroy(&self, window_label: &str) -> Result<()> {
        process::kill_mpv_process(&self.app, window_label)
    }

    pub fn command(
        &self,
        mpv_command: MpvCommand,
        window_label: &str,
    ) -> Result<MpvCommandResponse> {
        let ipc_timeout = {
            let instances_lock = self.app.mpv().instances.lock().unwrap();
            instances_lock.get(window_label).unwrap().ipc_timeout
        };
        ipc::send_command(mpv_command, window_label, ipc_timeout)
    }

    pub fn set_video_margin_ratio(
        &self,
        ratio: VideoMarginRatio,
        window_label: &str,
    ) -> Result<()> {
        let ipc_timeout = {
            let instances_lock = self.app.mpv().instances.lock().unwrap();
            instances_lock.get(window_label).unwrap().ipc_timeout
        };

        let margins = [
            ("video-margin-ratio-left", ratio.left),
            ("video-margin-ratio-right", ratio.right),
            ("video-margin-ratio-top", ratio.top),
            ("video-margin-ratio-bottom", ratio.bottom),
        ];

        for (property, value_option) in margins {
            if let Some(value) = value_option {
                let mpv_command = MpvCommand {
                    command: vec!["set_property".into(), property.into(), value.into()],
                    request_id: None,
                };
                ipc::send_command(mpv_command, window_label, ipc_timeout)?;
            }
        }

        Ok(())
    }
}
