use log::{debug, error, info, warn};
use std::{
    io::{BufRead, BufReader, Write},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Runtime};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::{ipc::get_ipc_pipe, process::kill_mpv_process, MpvEvent, MpvExt};

pub fn start_event_listener<R: Runtime>(
    app: &AppHandle<R>,
    process_id: u32,
    ipc_timeout: Duration,
    observed_properties: Vec<String>,
    window_label: &str,
) {
    let ipc_pipe = get_ipc_pipe(&window_label);

    let max_retries = 5;
    let mut retry_count = 0;

    loop {
        {
            let mut instances_lock = app.mpv().instances.lock().unwrap();

            if let Some(instance) = instances_lock.get_mut(window_label) {
                if instance.process.id() != process_id {
                    info!(
                        "mpv process for window '{}' has a different PID. Stopping listener.",
                        window_label
                    );
                    break;
                }
                if instance.process.try_wait().unwrap_or(None).is_some() {
                    info!(
                        "mpv process for window '{}' found terminated at start of loop. Stopping listener.",
                        window_label
                    );
                    break;
                }
            } else {
                info!(
                    "mpv process handle for window '{}' not found at start of loop. Stopping listener.",
                    window_label
                );
                break;
            }
        }

        retry_count += 1;

        debug!(
            "Event listener for window '{}' connecting... (attempt {}/{})",
            window_label, retry_count, max_retries
        );

        #[cfg(windows)]
        let stream_result = OpenOptions::new().read(true).write(true).open(&ipc_pipe);

        #[cfg(unix)]
        let stream_result = UnixStream::connect(&ipc_pipe);

        match stream_result {
            Ok(mut stream) => {
                info!(
                    "Event listener for window '{}' connected successfully.",
                    window_label
                );

                retry_count = 0;

                let mut successful_properties = Vec::new();
                let mut failed_properties = Vec::new();

                for (id, property) in observed_properties.iter().enumerate() {
                    let cmd_str = format!(
                        r#"{{"command": ["observe_property", {}, "{}"]}}"#,
                        id + 1,
                        property
                    );

                    let write_result = stream
                        .write_all(cmd_str.as_bytes())
                        .and_then(|_| stream.write_all(b"\n"))
                        .and_then(|_| stream.flush());

                    match write_result {
                        Ok(_) => {
                            successful_properties.push(property.clone());
                        }
                        Err(_) => {
                            failed_properties.push(property.clone());
                            break;
                        }
                    }
                }

                if !successful_properties.is_empty() {
                    info!(
                        "Successfully observed properties for window '{}': {:?}",
                        window_label, successful_properties
                    );
                }
                if !failed_properties.is_empty() {
                    warn!(
                        "Failed to observe properties for window '{}': {:?}",
                        window_label, failed_properties
                    );
                }

                let reader = BufReader::new(stream);
                for line_result in reader.lines() {
                    match line_result {
                        Ok(line) => {
                            if let Ok(payload) = serde_json::from_str::<MpvEvent>(&line) {
                                if payload.event.is_some() {
                                    let event_name = format!("mpv-event-{}", window_label);

                                    if let Err(e) =
                                        app.emit_to(&window_label, &event_name, &payload)
                                    {
                                        error!(
                                            "Failed to emit mpv event for window '{}': {}",
                                            window_label, e
                                        );
                                    }
                                }
                            } else {
                                warn!(
                                    "Failed to parse mpv event line as JSON for window '{}'. Line: '{}'",
                                    window_label, line,
                                );
                            }
                        }
                        Err(e) => {
                            error!(
                                "Error reading from mpv IPC on window '{}': {}",
                                window_label, e
                            );
                            break;
                        }
                    }
                }
                info!(
                    "mpv event listener for window '{}' has disconnected.",
                    window_label
                );
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            Err(e) => {
                debug!(
                    "Failed to connect to IPC for window '{}' (attempt {}/{}): {}",
                    window_label, retry_count, max_retries, e
                );

                if retry_count >= max_retries {
                    error!(
                        "Max retries reached for window '{}'. mpv IPC connection failed.",
                        window_label
                    );
                    break;
                }

                debug!("Retrying IPC connection for window '{}'...", window_label);
                std::thread::sleep(ipc_timeout);
            }
        }

        kill_mpv_process(app, window_label).unwrap_or_default();
    }
}
