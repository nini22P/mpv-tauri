use log::{debug, error, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Runtime};

use crate::events::{self, stop_event_listener};
use crate::ipc::get_ipc_pipe;

lazy_static::lazy_static! {
   pub static ref MPV_PROCESSES: Mutex<HashMap<String, Child>> = Mutex::new(HashMap::new());
}

pub fn init_mpv_process<R: Runtime>(
    app_handle: AppHandle<R>,
    window_handle: i64,
    mpv_config: HashMap<String, Value>,
    observed_properties: Vec<String>,
    window_label: &str,
) -> crate::Result<()> {
    let mut processes = MPV_PROCESSES.lock().unwrap();
    if let Some(child) = processes.get_mut(window_label) {
        match child.try_wait() {
            Ok(Some(_status)) => {
                warn!(
                    "Stale MPV process for window '{}' found and removed. Re-initializing...",
                    window_label
                );
                processes.remove(window_label);
            }
            Ok(None) => {
                info!(
                    "MPV process for window '{}' is still running. Skipping initialization.",
                    window_label
                );
                return Ok(());
            }
            Err(e) => {
                let error_message = format!(
                    "Failed to check status of existing MPV process for window '{}': {}",
                    window_label, e
                );
                error!("{}", error_message);
                return Err(crate::Error::MpvProcessError(error_message));
            }
        }
    }

    info!("Initializing MPV for window '{}'...", window_label);

    let ipc_pipe = get_ipc_pipe(&window_label);

    debug!("Using IPC pipe: {}", ipc_pipe);
    debug!("Starting MPV process with WID: {}", window_handle);

    // Default MPV arguments
    let mut args = vec![
        format!("--wid={}", window_handle),
        format!("--input-ipc-server={}", ipc_pipe),
        "--idle=yes".to_string(),
        "--force-window".to_string(),
        "--keep-open=yes".to_string(),
        "--no-border".to_string(),
        "--input-default-bindings=no".to_string(),
        "--input-vo-keyboard=no".to_string(),
        "--no-osc".to_string(),
    ];

    for (key, value) in mpv_config {
        let arg = match value {
            Value::String(s) => format!("--{}={}", key, s),
            Value::Number(n) => format!("--{}={}", key, n),
            Value::Bool(true) => format!("--{}", key),
            Value::Bool(false) => format!("--no-{}", key),
            _ => {
                warn!(
                    "For window '{}', unsupported config value type for key: '{}'. The setting will be ignored.",
                    window_label, key,
                );
                continue;
            }
        };
        args.push(arg);
    }

    debug!(
        "Spawning MPV process for window '{}' with args: mpv {}",
        window_label,
        args.join(" ")
    );

    let args_clone = args.clone();

    match Command::new("mpv")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            info!(
                "MPV process started for window '{}' (PID: {}). Initialization complete.",
                window_label,
                child.id()
            );
            processes.insert(window_label.to_string(), child);

            let window_label_clone = window_label.to_string();

            std::thread::spawn(move || {
                events::start_event_listener(app_handle, observed_properties, window_label_clone);
            });

            Ok(())
        }
        Err(e) => {
            let error_message = format!(
                "Failed to start MPV: {}. Is mpv installed and in your PATH?",
                e
            );
            error!("For window '{}': {}", window_label, error_message);
            debug!(
                "The command that failed for window '{}' was: mpv {}",
                window_label,
                args_clone.join(" ")
            );
            return Err(crate::Error::MpvProcessError(error_message));
        }
    }
}

pub fn kill_mpv_process(window_label: &str) -> crate::Result<()> {
    stop_event_listener(window_label);

    let mut processes = MPV_PROCESSES.lock().unwrap();

    if let Some(mut child) = processes.remove(window_label) {
        info!(
            "Attempting to kill MPV process for window '{}' (PID: {})...",
            window_label,
            child.id()
        );
        match child.kill() {
            Ok(_) => {
                let _ = child.wait();
                info!(
                    "MPV process for window '{}' killed successfully.",
                    window_label
                );
                Ok(())
            }
            Err(e) => {
                let error_message = format!(
                    "Failed to kill MPV process for window '{}': {}",
                    window_label, e
                );
                error!("{}", error_message);
                return Err(crate::Error::MpvProcessError(error_message));
            }
        }
    } else {
        info!(
            "No MPV process found for window '{}' to kill. It might have already been cleaned up.",
            window_label
        );
        Ok(())
    }
}
