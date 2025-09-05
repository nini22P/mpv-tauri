use std::io::{BufRead, BufReader, Read, Write};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::MpvCommandResponse;

#[cfg(windows)]
pub const IPC_PIPE_BASE: &str = r"\\.\pipe\tauri_plugin_mpv_socket";
#[cfg(unix)]
pub const IPC_PIPE_BASE: &str = "/tmp/tauri_plugin_mpv_socket";

pub fn get_ipc_pipe(window_label: &str) -> String {
    format!("{}_{}_{}", IPC_PIPE_BASE, std::process::id(), window_label)
}

pub fn send_command(command_json: &str, window_label: &str) -> crate::Result<MpvCommandResponse> {
    println!("[Tauri Plugin MPV][{}] SEND {}", window_label, command_json);

    let ipc_pipe = get_ipc_pipe(&window_label);

    #[cfg(windows)]
    {
        let pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&ipc_pipe)
            .map_err(|e| {
                crate::Error::IpcError(format!(
                    "[Tauri Plugin MPV][{}] Failed to open named pipe at '{}': {}",
                    window_label, ipc_pipe, e,
                ))
            })?;
        process_mpv_command(pipe, command_json, window_label)
    }

    #[cfg(unix)]
    {
        let stream = UnixStream::connect(&ipc_pipe).map_err(|e| {
            crate::Error::IpcError(format!(
                "[Tauri Plugin MPV][{}] Failed to connect to Unix socket at '{}': {}",
                window_label, ipc_pipe, e
            ))
        })?;
        process_mpv_command(stream, command_json, window_label)
    }
}

fn process_mpv_command<S: Read + Write>(
    mut stream: S,
    command_json: &str,
    window_label: &str,
) -> crate::Result<MpvCommandResponse> {
    stream.write_all(command_json.as_bytes()).map_err(|e| {
        crate::Error::IpcError(format!(
            "[Tauri Plugin MPV][{}] Failed to write command to IPC stream: {}",
            window_label, e,
        ))
    })?;
    stream.write_all(b"\n").map_err(|e| {
        crate::Error::IpcError(format!(
            "[Tauri Plugin MPV][{}] Failed to write newline to IPC stream: {}",
            window_label, e,
        ))
    })?;
    stream.flush().map_err(|e| {
        crate::Error::IpcError(format!(
            "[Tauri Plugin MPV][{}] Failed to flush IPC stream: {}",
            window_label, e,
        ))
    })?;

    let mut reader = BufReader::new(stream);
    let mut response_string = String::new();

    reader.read_line(&mut response_string).map_err(|e| {
        crate::Error::IpcError(format!(
            "[Tauri Plugin MPV][{}] Failed to read response from IPC stream: {}",
            window_label, e,
        ))
    })?;

    let response: MpvCommandResponse = serde_json::from_str(&response_string).map_err(|e| {
        crate::Error::IpcError(format!(
            "[Tauri Plugin MPV][{}] Failed to parse MPV response JSON: {}. Original response: '{}'",
            window_label, e, response_string,
        ))
    })?;

    println!(
        "[Tauri Plugin MPV][{}] RECV {}",
        window_label,
        serde_json::to_string(&response).unwrap_or_default()
    );

    Ok(response)
}
