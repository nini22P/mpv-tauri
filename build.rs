const COMMANDS: &[&str] = &["init", "destroy", "command", "set_video_margin_ratio"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
