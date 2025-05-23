#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tauri::Manager;

#[tauri::command]
fn send_mpv_command(command_json: &str) -> Result<String, String> {
    mpv_tauri_lib::send_mpv_command(command_json)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![send_mpv_command])
        .setup(|app| {
            let webview_window = app.get_webview_window("main").unwrap();
            let app_handle = app.handle().clone();
            let handle_result = webview_window.window_handle();

            match handle_result {
                Ok(handle_wrapper) => {
                    let raw_handle = handle_wrapper.as_raw();
                    let wid = match raw_handle {
                        RawWindowHandle::Win32(handle) => handle.hwnd.get() as i64,
                        RawWindowHandle::Xlib(handle) => handle.window as i64,
                        RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr() as i64,
                        _ => {
                            eprintln!("Unsupported window handle type for mpv --wid");
                            panic!("Unsupported window handle type");
                        }
                    };

                    std::thread::spawn(move || mpv_tauri_lib::init(wid));
                    std::thread::spawn(move || mpv_tauri_lib::mpv_event(app_handle));
                }
                Err(e) => {
                    eprintln!("Failed to get raw window handle: {:?}", e);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}