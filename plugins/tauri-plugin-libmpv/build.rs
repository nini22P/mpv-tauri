fn main() {
    tauri_plugin::Builder::new(&[
        "init",
        "destroy",
        "command",
        "set_property",
        "get_property",
        "set_video_margin_ratio",
    ])
    .android_path("android")
    .ios_path("ios")
    .build();
}
