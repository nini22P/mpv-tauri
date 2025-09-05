use std::io::{BufRead, BufReader, Read, Write};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::MpvCommandResponse;

#[cfg(windows)]
pub const IPC_PATH_BASE: &str = r"\\.\pipe\tauri_plugin_mpv_socket";
#[cfg(unix)]
pub const IPC_PATH_BASE: &str = "/tmp/tauri_plugin_mpv_socket";

pub fn get_ipc_path(window_label: &str) -> String {
    format!("{}_{}_{}", IPC_PATH_BASE, std::process::id(), window_label)
}

pub fn send_command(command_json: &str, window_label: &str) -> crate::Result<MpvCommandResponse> {
    println!("Tauri Plugin MPV: Received command: {}", command_json);

    let ipc_path = get_ipc_path(window_label);

    #[cfg(windows)]
    {
        let pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&ipc_path)
            .map_err(|e| {
                crate::Error::IpcError(format!(
                    "Failed to open named pipe at '{}': {}",
                    ipc_path, e
                ))
            })?;
        process_mpv_command(pipe, command_json)
    }

    #[cfg(unix)]
    {
        let stream = UnixStream::connect(&ipc_path).map_err(|e| {
            crate::Error::IpcError(format!(
                "Failed to connect to Unix socket at '{}': {}",
                ipc_path, e
            ))
        })?;
        process_mpv_command(stream, command_json)
    }
}

fn process_mpv_command<S: Read + Write>(
    mut stream: S,
    command_json: &str,
) -> crate::Result<MpvCommandResponse> {
    stream.write_all(command_json.as_bytes()).map_err(|e| {
        crate::Error::IpcError(format!("Failed to write command to IPC stream: {}", e))
    })?;
    stream.write_all(b"\n").map_err(|e| {
        crate::Error::IpcError(format!("Failed to write newline to IPC stream: {}", e))
    })?;
    stream
        .flush()
        .map_err(|e| crate::Error::IpcError(format!("Failed to flush IPC stream: {}", e)))?;

    let mut reader = BufReader::new(stream);
    let mut response_string = String::new();

    reader.read_line(&mut response_string).map_err(|e| {
        crate::Error::IpcError(format!("Failed to read response from IPC stream: {}", e))
    })?;

    let response: MpvCommandResponse =
        serde_json::from_str(&response_string.trim()).map_err(|e| {
            crate::Error::IpcError(format!(
                "Failed to parse MPV response JSON: {}. Original response: '{}'",
                e,
                response_string.trim()
            ))
        })?;

    println!(
        "Tauri Plugin MPV: Received response: {}",
        serde_json::to_string(&response).unwrap_or_default()
    );

    Ok(response)
}
