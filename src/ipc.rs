use std::io::{BufRead, BufReader, Read, Write};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use log::{error, trace};

use crate::MpvCommandResponse;
use crate::Result;

#[cfg(windows)]
pub const IPC_PIPE_BASE: &str = r"\\.\pipe\tauri_plugin_mpv_socket";
#[cfg(unix)]
pub const IPC_PIPE_BASE: &str = "/tmp/tauri_plugin_mpv_socket";

pub fn get_ipc_pipe(window_label: &str) -> String {
    format!("{}_{}_{}", IPC_PIPE_BASE, std::process::id(), window_label)
}

pub fn send_command(command_json: &str, window_label: &str) -> Result<MpvCommandResponse> {
    trace!("-> SEND [{}] {}", window_label, command_json);

    let ipc_pipe = get_ipc_pipe(&window_label);

    #[cfg(windows)]
    {
        let pipe = match OpenOptions::new().read(true).write(true).open(&ipc_pipe) {
            Ok(p) => p,
            Err(e) => {
                let err_msg = format!("Failed to open named pipe at '{}': {}", ipc_pipe, e);
                error!("For window '{}': {}", window_label, err_msg);
                return Err(crate::Error::IpcError(err_msg));
            }
        };
        process_mpv_command(pipe, command_json, window_label)
    }

    #[cfg(unix)]
    {
        let stream = match UnixStream::connect(&ipc_pipe) {
            Ok(s) => s,
            Err(e) => {
                let err_msg = format!("Failed to connect to Unix socket at '{}': {}", ipc_pipe, e);
                error!("For window '{}': {}", window_label, err_msg);
                return Err(crate::Error::IpcError(err_msg));
            }
        };
        process_mpv_command(stream, command_json, window_label)
    }
}

fn process_mpv_command<S: Read + Write>(
    mut stream: S,
    command_json: &str,
    window_label: &str,
) -> Result<MpvCommandResponse> {
    if let Err(e) = stream.write_all(command_json.as_bytes()) {
        let err_msg = format!("Failed to write command to IPC stream: {}", e);
        error!("For window '{}': {}", window_label, err_msg);
        return Err(crate::Error::IpcError(err_msg));
    }
    if let Err(e) = stream.write_all(b"\n") {
        let err_msg = format!("Failed to write newline to IPC stream: {}", e);
        error!("For window '{}': {}", window_label, err_msg);
        return Err(crate::Error::IpcError(err_msg));
    }
    if let Err(e) = stream.flush() {
        let err_msg = format!("Failed to flush IPC stream: {}", e);
        error!("For window '{}': {}", window_label, err_msg);
        return Err(crate::Error::IpcError(err_msg));
    }

    let mut reader = BufReader::new(stream);
    let mut response_string = String::new();

    if let Err(e) = reader.read_line(&mut response_string) {
        let err_msg = format!("Failed to read response from IPC stream: {}", e);
        error!("For window '{}': {}", window_label, err_msg);
        return Err(crate::Error::IpcError(err_msg));
    }

    let response: MpvCommandResponse = match serde_json::from_str(&response_string) {
        Ok(res) => res,
        Err(e) => {
            let err_msg = format!(
                "Failed to parse MPV response JSON: {}. Original response: '{}'",
                e,
                response_string.trim_end()
            );
            error!("For window '{}': {}", window_label, err_msg);
            return Err(crate::Error::IpcError(err_msg));
        }
    };

    trace!(
        "<- RECV [{}] {}",
        window_label,
        serde_json::to_string(&response).unwrap_or_default()
    );

    Ok(response)
}
