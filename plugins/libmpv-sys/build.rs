fn main() {
    println!("cargo:rustc-link-lib=mpv");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "generate-bindings")]
    generate_bindings();

    #[cfg(not(feature = "generate-bindings"))]
    println!("cargo:rerun-if-changed=src/bindings.rs");
}

#[cfg(feature = "generate-bindings")]
fn generate_bindings() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    println!("Starting binding generation for libmpv...");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let base_url = "https://github.com/mpv-player/mpv/raw/refs/heads/master/include/mpv/";
    let headers = ["client.h", "render.h", "render_gl.h", "stream_cb.h"];
    let mut wrapper_content = String::new();

    println!("Downloading C header files from mpv repository...");
    for header_name in &headers {
        let header_path = out_path.join(header_name);

        println!("Downloading {}...", header_name);
        let url = format!("{}{}", base_url, header_name);
        let header_content = reqwest::blocking::get(&url)
            .unwrap_or_else(|_| panic!("Failed to get {}", url))
            .text()
            .unwrap_or_else(|_| panic!("Failed to get text from {}", url));
        fs::write(&header_path, &header_content).expect("Failed to write header");

        wrapper_content.push_str(&format!("#include \"{}\"\n", header_name));
    }

    let wrapper_path = out_path.join("wrapper.h");
    fs::write(&wrapper_path, &wrapper_content).expect("Failed to write wrapper.h");
    println!("Header files downloaded and wrapper.h created.");

    let clang_arg = format!("-I{}", out_path.display());

    println!("Running bindgen to generate Rust bindings...");
    let bindings = bindgen::Builder::default()
        .formatter(bindgen::Formatter::Prettyplease)
        .header(wrapper_path.to_str().unwrap())
        .clang_arg(clang_arg)
        .impl_debug(true)
        .opaque_type("mpv_handle")
        .opaque_type("mpv_render_context")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let bindings_path = manifest_dir.join("src").join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings!");

    println!(
        "Successfully generated bindings at: {}",
        bindings_path.display()
    );
}
