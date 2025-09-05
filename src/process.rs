use serde_json::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::ipc::get_ipc_pipe;

pub fn init_mpv_process(
    window_handle: i64,
    window_label: &str,
    mpv_config: Option<HashMap<String, Value>>,
) -> crate::Result<()> {
    let ipc_pipe = get_ipc_pipe(window_label);

    // Default MPV arguments
    let mut args = vec![
        format!("--wid={}", window_handle),
        format!("--input-ipc-server={}", ipc_pipe),
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
                        "[Tauri Plugin MPV] Unsupported config value type for key: {}",
                        key
                    );
                    continue;
                }
            };
            args.push(arg);
        }
    }

    println!("[Tauri Plugin MPV] mpv {}", &args.join(" "));

    let args_clone = args.clone();
    match Command::new("mpv")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            println!(
                "[Tauri Plugin MPV] MPV process started (PID: {}). Initialization complete",
                child.id()
            );
            Ok(())
        }
        Err(e) => {
            let error_message = format!(
                "[Tauri Plugin MPV] Failed to start MPV: {}. Is mpv installed and in your PATH?",
                e
            );
            eprintln!("{}", error_message);
            eprintln!("[Tauri Plugin MPV] mpv {}", args_clone.join(" "));
            Err(crate::Error::MpvProcessError(error_message))
        }
    }
}
