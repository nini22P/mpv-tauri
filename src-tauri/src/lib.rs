use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
pub const IPC_PATH: &str = r"\\.\pipe\mpvsocket";
#[cfg(unix)]
pub const IPC_PATH: &str = "/tmp/mpvsocket";

#[derive(Debug, Serialize, Deserialize)]
pub struct MpvCommand {
    command: Vec<String>,
}

pub fn send_mpv_command(command_json: &str) -> Result<String, String> {
    println!("Received command: {}", command_json);

    #[cfg(windows)]
    {
        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(IPC_PATH)
            .map_err(|e| format!("Failed to open named pipe at '{}': {}", IPC_PATH, e))?;

        pipe.write_all(command_json.as_bytes())
            .map_err(|e| format!("Failed to write command to named pipe: {}", e))?;

        pipe.write_all(b"\n")
            .map_err(|e| format!("Failed to write newline to named pipe: {}", e))?;

        pipe.flush()
            .map_err(|e| format!("Failed to flush named pipe: {}", e))?;

        let mut reader = BufReader::new(pipe);
        let mut response = String::new();

        reader
            .read_line(&mut response)
            .map_err(|e| format!("Failed to read response from named pipe: {}", e))?;

        println!("Received response: {}", response);

        Ok(response)
    }

    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(IPC_PATH)
            .map_err(|e| format!("Failed to connect to Unix socket at '{}': {}", IPC_PATH, e))?;

        stream
            .write_all(command_json.as_bytes())
            .map_err(|e| format!("Failed to write command to Unix socket: {}", e))?;

        stream
            .write_all(b"\n")
            .map_err(|e| format!("Failed to write newline to Unix socket: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();

        reader
            .read_line(&mut response)
            .map_err(|e| format!("Failed to read response from Unix socket: {}", e))?;

        println!("Received response: {}", response);

        Ok(response)
    }
}