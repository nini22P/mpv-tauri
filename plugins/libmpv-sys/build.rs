fn main() {
    // Link to the mpv library.
    // Cargo will search for `mpv.lib` on Windows, `libmpv.so` on Linux, and `libmpv.dylib` on macOS.
    // The user of the final Tauri app will need to have this library installed or bundled with the app.
    println!("cargo:rustc-link-lib=mpv");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bindings.rs");
}
