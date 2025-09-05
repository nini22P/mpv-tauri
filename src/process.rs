use serde_json::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::ipc::get_ipc_path;

pub fn init_mpv_process(
    window_handle: i64,
    window_label: &str,
    mpv_config: Option<HashMap<String, Value>>,
) -> crate::Result<()> {
    println!(
        "Tauri Plugin MPV: Attempting to start mpv with WID: {} for window: {}",
        window_handle, window_label
    );

    let ipc_path = get_ipc_path(window_label);
    println!("Tauri Plugin MPV: Using IPC path: {}", ipc_path);

    // Default MPV arguments
    let mut args = vec![
        format!("--wid={}", window_handle),
        format!("--input-ipc-server={}", ipc_path),
        "--idle=yes".to_string(),
        "--force-window".to_string(),
        "--keep-open=yes".to_string(),
        "--no-border".to_string(),
        "--input-default-bindings=no".to_string(),
        "--input-vo-keyboard=no".to_string(),
        "--no-osc".to_string(),
    ];

    if let Some(config) = mpv_config {
        for (key, value) in config {
            let arg = match value {
                Value::String(s) => format!("--{}={}", key, s),
                Value::Number(n) => format!("--{}={}", key, n),
                Value::Bool(true) => format!("--{}", key),
                Value::Bool(false) => format!("--no-{}", key),
                _ => {
                    println!(
                        "Tauri Plugin MPV: Unsupported config value type for key: {}",
                        key
                    );
                    continue;
                }
            };
            args.push(arg);
            println!(
                "Tauri Plugin MPV: Added config option: {}",
                args.last().unwrap()
            );
        }
    }

    println!("Tauri Plugin MPV: mpv {}", args.join(" "));

    let args_clone = args.clone();
    match Command::new("mpv")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            println!(
                "Tauri Plugin MPV: MPV process started successfully with PID: {}",
                child.id()
            );
            Ok(())
        }
        Err(e) => {
            let error_message = format!(
                "Tauri Plugin MPV: Failed to start mpv: {}. Is mpv installed and in your PATH?",
                e
            );
            eprintln!("{}", error_message);
            eprintln!(
                "Tauri Plugin MPV: Attempted command: mpv {}",
                args_clone.join(" ")
            );
            Err(crate::Error::MpvProcessError(error_message))
        }
    }
}
