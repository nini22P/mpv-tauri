use log::{debug, error, info, trace, warn};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Runtime};

use crate::events::{self, stop_event_listener};
use crate::ipc::get_ipc_pipe;
use crate::MpvConfig;

lazy_static::lazy_static! {
   pub static ref MPV_PROCESSES: Mutex<HashMap<String, Child>> = Mutex::new(HashMap::new());
}

pub fn init_mpv_process<R: Runtime>(
    app_handle: AppHandle<R>,
    window_handle: i64,
    mpv_config: MpvConfig,
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
    debug!(
        "Starting MPV process for window '{}' (WID: {})",
        window_label, window_handle
    );

    // Default MPV arguments
    // libmpv profile: https://github.com/mpv-player/mpv/blob/master/etc/builtin.conf#L21
    let mut args = vec![
        format!("--wid={}", window_handle),
        format!("--input-ipc-server={}", ipc_pipe),
        "--profile=libmpv".to_string(),
        "--force-window".to_string(),
        "--keep-open=yes".to_string(),
        "--border=no".to_string(),
    ];

    args.extend(mpv_config.mpv_args.unwrap_or_default());

    let mpv_path = mpv_config.mpv_path.unwrap_or_else(|| "mpv".to_string());

    debug!(
        "Spawning MPV process for window '{}' with args: {} {}",
        window_label,
        mpv_path,
        args.join(" ")
    );

    let args_clone = args.clone();
    let ipc_timeout = Duration::from_millis(mpv_config.ipc_timeout_ms.unwrap_or(2000));
    let show_mpv_output = mpv_config.show_mpv_output.unwrap_or(false);

    match Command::new(mpv_path.clone())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            let log_queue: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
            let log_queue_clone = Arc::clone(&log_queue);
            let window_label_clone = window_label.to_string();

            if let Some(stdout) = child.stdout.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().flatten() {
                        if show_mpv_output {
                            trace!("MPV stdout [{}] {}", window_label_clone, line);
                        }
                        if let Ok(mut queue) = log_queue_clone.lock() {
                            queue.push_back(line);
                            if queue.len() > 100 {
                                queue.pop_front();
                            }
                        }
                    }
                });
            }

            match wait_for_ipc_server(&ipc_pipe, ipc_timeout, window_label) {
                Ok(startup_duration) => {
                    info!(
                        "MPV IPC server for window '{}' is ready. Startup took {}ms.",
                        window_label,
                        startup_duration.as_millis()
                    );
                }
                Err(e) => {
                    let mut error_message = format!(
                        "MPV startup failed for window '{}'. Collected stdout:",
                        window_label,
                    );
                    error!("{}", error_message);
                    error_message.push_str("\n");
                    if let Ok(mut queue) = log_queue.lock() {
                        while let Some(line) = queue.pop_front() {
                            error!("MPV stdout [{}] {}", window_label, line);
                            error_message
                                .push_str(&format!("MPV stdout [{}] {}\n", window_label, line));
                        }
                    }
                    error!("{}", e);
                    error_message.push_str(&e);
                    let _ = child.kill();
                    return Err(crate::Error::MpvProcessError(error_message));
                }
            }

            info!(
                "MPV process (PID: {}) started for window '{}'. Initialization complete.",
                child.id(),
                window_label,
            );

            processes.insert(window_label.to_string(), child);

            let window_label_clone = window_label.to_string();

            let observed_properties = mpv_config.observed_properties.clone().unwrap_or_default();

            std::thread::spawn(move || {
                events::start_event_listener(
                    app_handle,
                    ipc_timeout,
                    observed_properties,
                    &window_label_clone,
                );
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
                "The command that failed for window '{}' was: {} {}",
                window_label,
                mpv_path,
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

fn wait_for_ipc_server(
    ipc_pipe: &str,
    ipc_timeout: Duration,
    window_label: &str,
) -> Result<Duration, String> {
    let start = Instant::now();
    let pipe = Path::new(ipc_pipe);

    while start.elapsed() < ipc_timeout {
        if pipe.exists() {
            let elapsed = start.elapsed();
            debug!(
                "IPC server for window '{}' at '{}' is ready after {:?}",
                window_label, ipc_pipe, elapsed
            );
            return Ok(elapsed);
        }
        thread::sleep(Duration::from_millis(50));
    }

    Err(format!(
        "Timed out after {:?} waiting for IPC server for window '{}' at '{}'",
        ipc_timeout, window_label, ipc_pipe,
    ))
}
