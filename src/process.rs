use log::{debug, error, info, trace, warn};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Runtime};

use crate::events::{self};
use crate::ipc::get_ipc_pipe;
use crate::{MpvConfig, MpvExt, MpvInstance};

pub fn init_mpv_process<R: Runtime>(
    app: &AppHandle<R>,
    window_handle: i64,
    mpv_config: MpvConfig,
    window_label: &str,
) -> crate::Result<()> {
    let ipc_pipe = get_ipc_pipe(&window_label);
    let ipc_timeout = Duration::from_millis(mpv_config.ipc_timeout_ms.unwrap_or(2000));

    let mut instances_lock = app.mpv().instances.lock().unwrap();
    if let Some(instance) = instances_lock.get_mut(window_label) {
        if instance.process.try_wait().unwrap_or(None).is_none() {
            match wait_for_ipc_server(&ipc_pipe, ipc_timeout, window_label) {
                Ok(_) => {
                    info!(
                        "mpv process (PID: {}) for window '{}' is still running. Skipping initialization.",
                        instance.process.id(),
                        window_label
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to IPC for window '{}': {}, killing and restarting...",
                        window_label, e
                    );
                    instance.process.kill().unwrap();
                }
            }
        }
    }

    info!("Initializing mpv for window '{}'...", window_label);

    debug!("Using IPC pipe: {}", ipc_pipe);
    debug!(
        "Starting mpv process for window '{}' (WID: {})",
        window_label, window_handle
    );

    // Default mpv arguments
    // libmpv profile: https://github.com/mpv-player/mpv/blob/master/etc/builtin.conf#L21
    let mut args = vec![
        format!("--wid={}", window_handle),
        format!("--input-ipc-server={}", ipc_pipe),
        "--profile=libmpv".to_string(),
    ];

    args.extend(mpv_config.mpv_args.unwrap_or_default());

    let mpv_path = mpv_config.mpv_path.unwrap_or_else(|| "mpv".to_string());

    debug!(
        "Spawning mpv process for window '{}' with args: {} {}",
        window_label,
        mpv_path,
        args.join(" ")
    );

    let args_clone = args.clone();
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
                            trace!("mpv stdout [{}] {}", window_label_clone, line);
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
                        "mpv IPC server for window '{}' is ready. Startup took {}ms.",
                        window_label,
                        startup_duration.as_millis()
                    );
                }
                Err(e) => {
                    let mut error_message = format!(
                        "mpv startup failed for window '{}'. Collected stdout:",
                        window_label,
                    );
                    error!("{}", error_message);
                    error_message.push_str("\n");
                    if let Ok(mut queue) = log_queue.lock() {
                        while let Some(line) = queue.pop_front() {
                            error!("mpv stdout [{}] {}", window_label, line);
                            error_message
                                .push_str(&format!("mpv stdout [{}] {}\n", window_label, line));
                        }
                    }
                    error!("{}", e);
                    error_message.push_str(&e);
                    let _ = child.kill();
                    return Err(crate::Error::MpvProcessError(error_message));
                }
            }

            info!(
                "mpv process (PID: {}) started for window '{}'. Initialization complete.",
                child.id(),
                window_label,
            );

            let window_label_clone = window_label.to_string();
            let observed_properties = mpv_config.observed_properties.clone().unwrap_or_default();
            let app_clone = app.clone();
            let process_id = child.id();

            let instance = MpvInstance { process: child };
            instances_lock.insert(window_label.to_string(), instance);

            std::thread::spawn(move || {
                events::start_event_listener(
                    &app_clone,
                    process_id,
                    ipc_timeout,
                    observed_properties,
                    &window_label_clone,
                );
            });

            Ok(())
        }
        Err(e) => {
            let error_message = format!(
                "Failed to start mpv: {}. Is mpv installed and in your PATH?",
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

pub fn kill_mpv_process<R: Runtime>(app: &AppHandle<R>, window_label: &str) -> crate::Result<()> {
    let instance_to_kill = {
        let mut instances_lock = app.mpv().instances.lock().unwrap();
        instances_lock.remove(window_label)
    };

    if let Some(mut instance) = instance_to_kill {
        info!(
            "Attempting to kill mpv process (PID: {}) for window '{}'...",
            instance.process.id(),
            window_label,
        );
        match instance.process.kill() {
            Ok(_) => {
                let _ = instance.process.wait();
                info!(
                    "mpv process (PID: {}) for window '{}' killed successfully.",
                    instance.process.id(),
                    window_label,
                );
                Ok(())
            }
            Err(e) => {
                let error_message = format!(
                    "Failed to kill mpv process (PID: {}) for window '{}': {}",
                    instance.process.id(),
                    window_label,
                    e,
                );
                error!("{}", error_message);
                return Err(crate::Error::MpvProcessError(error_message));
            }
        }
    } else {
        info!(
            "No running mpv process found for window '{}' to kill. It may have already terminated.",
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
            return Ok(elapsed);
        }
        thread::sleep(Duration::from_millis(50));
    }

    Err(format!(
        "Timed out after {:?} waiting for IPC server for window '{}' at '{}'",
        ipc_timeout, window_label, ipc_pipe,
    ))
}
