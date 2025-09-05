const COMMANDS: &[&str] = &[
    "initialize_mpv",
    "destroy_mpv",
    "send_mpv_command",
    "set_video_margin_ratio",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
