use std::{
    io::{BufRead, BufReader, Write},
    process::Command,
    time::Duration,
};
use tauri::Emitter;

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
pub const IPC_PATH: &str = r"\\.\pipe\mpvsocket";
#[cfg(unix)]
pub const IPC_PATH: &str = "/tmp/mpvsocket";

pub static IPC_PATH_ONCELOCK: std::sync::OnceLock<String> = std::sync::OnceLock::new();

#[derive(Clone, serde::Serialize)]
pub struct MpvEvent {
    event_type: String,
    name: Option<String>,
    data: Option<serde_json::Value>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct VideoMarginRatio {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

pub const OBSERVED_PROPERTIES: [&str; 6] = [
    "playlist",
    "filename",
    "pause",
    "eof-reached",
    "time-pos",
    "duration",
];

pub fn set_ipc_path(window_handle: i64) {
    let ipc_path = format!("{}_{}", IPC_PATH, window_handle);
    println!("Setting IPC Path: {}", ipc_path);
    IPC_PATH_ONCELOCK.set(ipc_path).unwrap();
}

pub fn init(window_handle: i64) {
    println!("Attempting to start mpv with WID: {}", window_handle);

    let ipc_path = IPC_PATH_ONCELOCK.get().unwrap();

    Command::new("mpv")
        .args(&[
            &format!("--wid={}", window_handle),
            &format!("--input-ipc-server={}", ipc_path),
            "--idle=yes",
            "--force-window",
            "--keep-open=yes",
            "--no-border",
            "--input-default-bindings=no",
            "--input-vo-keyboard=no",
            "--no-osc",
        ])
        .spawn()
        .expect("Failed to start mpv. Is mpv installed and in your PATH?");
}

pub fn send_mpv_command(command_json: &str) -> Result<String, String> {
    let ipc_path = IPC_PATH_ONCELOCK.get().unwrap();

    #[cfg(windows)]
    {
        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(ipc_path)
            .map_err(|e| format!("Failed to open named pipe at '{}': {}", ipc_path, e))?;

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
        let mut stream = UnixStream::connect(ipc_path)
            .map_err(|e| format!("Failed to connect to Unix socket at '{}': {}", ipc_path, e))?;

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

pub fn mpv_event(app_handle: tauri::AppHandle) {
    let ipc_path = IPC_PATH_ONCELOCK.get().unwrap();

    {
        std::thread::sleep(Duration::from_secs(2));

        #[cfg(windows)]
        let stream_result = OpenOptions::new().read(true).write(true).open(ipc_path);

        #[cfg(unix)]
        let stream_result = UnixStream::connect(ipc_path);

        match stream_result {
            Ok(mut stream) => {
                println!("Successfully connected to mpv IPC for event listening.");

                let observe_commands: Vec<String> = OBSERVED_PROPERTIES
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
                            // println!("MPV Event: {}", line);
                            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&line)
                            {
                                if let Some(event_name_val) = json_value.get("event") {
                                    let event_name = event_name_val.as_str().unwrap_or_default();

                                    let mut payload_name: Option<String> = None;
                                    if let Some(name_val) = json_value.get("name") {
                                        payload_name = name_val.as_str().map(String::from);
                                    }

                                    let payload_data = json_value.get("data").cloned();

                                    let payload = MpvEvent {
                                        event_type: event_name.to_string(),
                                        name: payload_name,
                                        data: payload_data,
                                    };
                                    app_handle.emit_to("main", "mpv-event", &payload).unwrap();
                                }
                            } else {
                                eprintln!("Failed to parse mpv event line as JSON: {}", line);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from mpv IPC: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to connect to mpv IPC for event listening at '{}': {}",
                    ipc_path, e
                );
            }
        }
    }
}

pub fn set_video_margin_ratio(ratio: VideoMarginRatio) {
    let command = format!(
        r#"{{"command": ["set_property", "video-margin-ratio-left", {}]}}"#,
        ratio.left,
    );

    send_mpv_command(&command).unwrap();

    let command = format!(
        r#"{{"command": ["set_property", "video-margin-ratio-right", {}]}}"#,
        ratio.right,
    );

    send_mpv_command(&command).unwrap();

    let command = format!(
        r#"{{"command": ["set_property", "video-margin-ratio-top", {}]}}"#,
        ratio.top,
    );

    send_mpv_command(&command).unwrap();

    let command = format!(
        r#"{{"command": ["set_property", "video-margin-ratio-bottom", {}]}}"#,
        ratio.bottom,
    );

    send_mpv_command(&command).unwrap();
}
