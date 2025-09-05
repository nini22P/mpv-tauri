use std::{
    io::{BufRead, BufReader, Write},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Runtime};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::{ipc::get_ipc_pipe, MpvEvent};

pub fn start_event_listener<R: Runtime>(
    app_handle: AppHandle<R>,
    properties: Vec<String>,
    window_label: String,
) {
    let ipc_pipe = get_ipc_pipe(&window_label);

    let max_retries = 5;
    let mut retry_count = 0;

    loop {
        retry_count += 1;

        println!(
            "[Tauri Plugin MPV] Event listener connecting... (attempt {}/{})",
            retry_count, max_retries
        );

        std::thread::sleep(Duration::from_secs(1));

        #[cfg(windows)]
        let stream_result = OpenOptions::new().read(true).write(true).open(&ipc_pipe);

        #[cfg(unix)]
        let stream_result = UnixStream::connect(&ipc_pipe);

        match stream_result {
            Ok(mut stream) => {
                println!("[Tauri Plugin MPV] Event listener connected successfully.");

                retry_count = 0;

                for (id, property) in properties.iter().enumerate() {
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
                            // println!("[Tauri Plugin MPV] Successfully observed property: {}", property);
                        }
                        Err(e) => {
                            eprintln!(
                                "[Tauri Plugin MPV] Failed to observe property '{}': {}",
                                property, e
                            );
                            break;
                        }
                    }
                }

                let reader = BufReader::new(stream);
                for line_result in reader.lines() {
                    match line_result {
                        Ok(line) => {
                            if let Ok(payload) = serde_json::from_str::<MpvEvent>(&line) {
                                if payload.event.is_some() {
                                    let event_name = format!("mpv-event-{}", window_label);

                                    if let Err(e) =
                                        app_handle.emit_to(&window_label, &event_name, &payload)
                                    {
                                        eprintln!(
                                            "[Tauri Plugin MPV] Failed to emit MPV event to window '{}': {}",
                                            window_label, e
                                        );
                                    }
                                }
                            } else {
                                eprintln!(
                                    "[Tauri Plugin MPV] Failed to parse MPV event line as JSON: {}",
                                    line
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("[Tauri Plugin MPV] Error reading from MPV IPC: {}", e);
                            break;
                        }
                    }
                }
                println!(
                    "[Tauri Plugin MPV] MPV event listener disconnected for window '{}'.",
                    window_label
                );
            }
            Err(e) => {
                eprintln!(
                    "[Tauri Plugin MPV] Failed to connect to mpv IPC for event listening at '{}' (attempt {}/{}): {}",
                    ipc_pipe, retry_count, max_retries, e
                );

                if retry_count >= max_retries {
                    eprintln!("[Tauri Plugin MPV] Max retries reached. MPV IPC connection failed.");
                    break;
                }

                println!("[Tauri Plugin MPV] Retrying...");
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
