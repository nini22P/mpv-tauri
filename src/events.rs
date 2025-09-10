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

use crate::{ipc::get_ipc_pipe, MpvEvent, MpvExt};

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
                if process_id != instance.process.id() {
                    info!(
                        "Stopping stale listener for old PID {} for window '{}', detected new mpv process (PID {}).",
                        process_id,
                        window_label,
                        instance.process.id(),
                    );
                    break;
                }
                if instance.process.try_wait().unwrap_or(None).is_some() {
                    info!(
                        "Stopping listener for associated mpv process (PID {}) for window '{}', as the process has terminated.",
                        instance.process.id(),
                        window_label,
                    );
                    break;
                }
            } else {
                info!(
                    "Instance for window '{}' not found. Stopping listener for mpv process (PID: {}) .",
                    window_label, process_id,
                );
                break;
            }
        }

        retry_count += 1;

        debug!(
            "Event listener for mpv process (PID: {}) for window '{}' connecting... (attempt {}/{})",
           process_id, window_label, retry_count, max_retries,
        );

        #[cfg(windows)]
        let stream_result = OpenOptions::new().read(true).write(true).open(&ipc_pipe);

        #[cfg(unix)]
        let stream_result = UnixStream::connect(&ipc_pipe);

        match stream_result {
            Ok(mut stream) => {
                info!(
                    "Successfully connected event listener for mpv process (PID: {}) for window '{}'.",
                    process_id,
                    window_label,
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
                        "Successfully observed properties for mpv process (PID: {}) for window '{}': {:?}",
                        process_id,
                        window_label, 
                        successful_properties,
                    );
                }
                if !failed_properties.is_empty() {
                    warn!(
                        "Failed to observe properties for mpv process (PID: {}) for window '{}': {:?}",
                        process_id, window_label, failed_properties
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
                                            "Failed to emit mpv event for mpv process (PID: {}) for window '{}': {}",
                                            process_id,
                                            window_label,
                                            e,
                                        );
                                    }
                                }
                            } else {
                                warn!(
                                    "Failed to parse mpv event line as JSON for mpv process (PID: {}) for window '{}'. Line: '{}'",
                                    process_id,
                                    window_label,
                                    line,
                                );
                            }
                        }
                        Err(e) => {
                            error!(
                                "Error reading from mpv IPC for mpv process (PID: {}) for window '{}': {}",
                                process_id,
                                window_label,
                                e,
                            );
                            break;
                        }
                    }
                }
                info!(
                    "Event listener for mpv process (PID: {}) for window '{}' has disconnected.",
                    process_id, window_label,
                );
                std::thread::sleep(ipc_timeout);
                continue;
            }
            Err(e) => {
                debug!(
                    "Failed to connect to IPC for mpv process (PID: {}) for window '{}' (attempt {}/{}): {}",
                    process_id,
                    window_label,
                    retry_count,
                    max_retries,
                    e,
                );

                if retry_count >= max_retries {
                    error!(
                        "Max retries reached for mpv process (PID: {}) for window '{}'. mpv IPC connection failed.",
                        process_id, window_label,
                    );
                    break;
                }

                debug!(
                    "Retrying IPC connection for mpv process (PID: {}) for window '{}'...",
                    process_id, window_label,
                );
                std::thread::sleep(ipc_timeout);
            }
        }
    }
}
