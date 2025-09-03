use std::io::{BufRead, BufReader, Write};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
pub const IPC_PATH_BASE: &str = r"\\.\pipe\tauri_plugin_mpv_socket";
#[cfg(unix)]
pub const IPC_PATH_BASE: &str = "/tmp/tauri_plugin_mpv_socket";

pub fn get_ipc_path(window_label: &str) -> String {
    format!("{}_{}", IPC_PATH_BASE, window_label)
}

pub fn send_command(command_json: &str, window_label: &str) -> crate::Result<String> {
    println!("Received command: {}", command_json);

    let ipc_path = get_ipc_path(window_label);
    println!("Using IPC path: {}", ipc_path);

    #[cfg(windows)]
    {
        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&ipc_path)
            .map_err(|e| {
                crate::Error::IpcError(format!(
                    "Failed to open named pipe at '{}': {}",
                    ipc_path, e
                ))
            })?;

        pipe.write_all(command_json.as_bytes()).map_err(|e| {
            crate::Error::IpcError(format!("Failed to write command to named pipe: {}", e))
        })?;

        pipe.write_all(b"\n").map_err(|e| {
            crate::Error::IpcError(format!("Failed to write newline to named pipe: {}", e))
        })?;

        pipe.flush()
            .map_err(|e| crate::Error::IpcError(format!("Failed to flush named pipe: {}", e)))?;

        let mut reader = BufReader::new(pipe);
        let mut response = String::new();

        reader.read_line(&mut response).map_err(|e| {
            crate::Error::IpcError(format!("Failed to read response from named pipe: {}", e))
        })?;

        println!("Received response: {}", response);

        Ok(response)
    }

    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(&ipc_path).map_err(|e| {
            crate::Error::IpcError(format!(
                "Failed to connect to Unix socket at '{}': {}",
                ipc_path, e
            ))
        })?;

        stream.write_all(command_json.as_bytes()).map_err(|e| {
            crate::Error::IpcError(format!("Failed to write command to Unix socket: {}", e))
        })?;

        stream.write_all(b"\n").map_err(|e| {
            crate::Error::IpcError(format!("Failed to write newline to Unix socket: {}", e))
        })?;

        stream
            .flush()
            .map_err(|e| crate::Error::IpcError(format!("Failed to flush Unix socket: {}", e)))?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();

        reader.read_line(&mut response).map_err(|e| {
            crate::Error::IpcError(format!("Failed to read response from Unix socket: {}", e))
        })?;

        println!("Received response: {}", response);

        Ok(response)
    }
}
