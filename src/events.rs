use std::{
    io::{BufRead, BufReader, Write},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Runtime};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::ipc::get_ipc_path;

pub fn start_event_listener<R: Runtime>(
    app_handle: AppHandle<R>,
    properties: Vec<String>,
    window_label: String,
) {
    let ipc_path = get_ipc_path(&window_label);
    println!("Event listener using IPC path: {}", ipc_path);

    println!("Waiting for MPV IPC server to be ready...");
    std::thread::sleep(Duration::from_secs(3));

    let max_retries = 5;
    let mut retry_count = 0;

    loop {
        retry_count += 1;
        println!(
            "Attempting to connect to MPV IPC (attempt {}/{})",
            retry_count, max_retries
        );

        #[cfg(windows)]
        let stream_result = OpenOptions::new().read(true).write(true).open(&ipc_path);

        #[cfg(unix)]
        let stream_result = UnixStream::connect(&ipc_path);

        match stream_result {
            Ok(mut stream) => {
                println!("Successfully connected to mpv IPC for event listening.");

                let observe_commands: Vec<String> = properties
                    .iter()
                    .enumerate()
                    .map(|(id, property)| {
                        format!(
                            r#"{{"command": ["observe_property", {}, "{}"]}}"#,
                            id + 1,
                            property
                        )
                    })
                    .collect();

                for cmd_str in observe_commands.iter() {
                    if stream.write_all(cmd_str.as_bytes()).is_ok()
                        && stream.write_all(b"\n").is_ok()
                    {
                        stream.flush().ok();
                        println!("Sent: {}", cmd_str);
                    } else {
                        eprintln!("Failed to send: {}", cmd_str);
                    }
                }

                let reader = BufReader::new(stream);
                for line_result in reader.lines() {
                    match line_result {
                        Ok(line) => {
                            let event_name = format!("mpv-event-{}", window_label);
                            if let Err(e) = app_handle.emit_to(&window_label, &event_name, &line) {
                                eprintln!(
                                    "Failed to emit MPV event to window '{}': {}",
                                    window_label, e
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from mpv IPC: {}", e);
                            break;
                        }
                    }
                }
                println!(
                    "MPV event listener disconnected for window '{}'.",
                    window_label
                );
            }
            Err(e) => {
                eprintln!(
                    "Failed to connect to mpv IPC for event listening at '{}' (attempt {}/{}): {}",
                    ipc_path, retry_count, max_retries, e
                );

                if retry_count >= max_retries {
                    eprintln!("Max retries reached. MPV IPC connection failed.");
                    break;
                }

                println!("Retrying in 2 seconds...");
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
