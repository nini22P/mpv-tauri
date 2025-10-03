# tauri-plugin-libmpv-sys

Raw FFI bindings for `libmpv`.

This crate ships with pre-generated bindings for the latest version of `libmpv`.

## Generating Bindings

If you need to regenerate the bindings against a different version of `libmpv` or for a different target, you can do so by enabling the `generate-bindings` feature.

This requires `clang` to be installed.

Run the following command:

```bash
cargo build --features "generate-bindings"
