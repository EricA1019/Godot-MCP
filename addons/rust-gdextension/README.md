# Rust GDExtension (optional)

This extension is disabled by default to prevent startup errors when the native library isn’t built.

Enable steps
1) Build the native lib for your platform (from repo root):
   - Debug: the library should land under `.rust/target/debug/` (paths referenced in the descriptor)
   - Release: `.rust/target/release/`
2) Rename `rust.gdextension.disabled` to `rust.gdextension`.
3) Open Godot; ensure the plugin loads. If needed, set Cargo paths via the `Rust Auto Compile` plugin (addons/rust_auto_compile).

Troubleshooting
- Error: dynamic library not found
  - Verify the file exists at the path from `addons/rust-gdextension/rust.gdextension`
  - Example on Linux (debug): `.rust/target/debug/librust_gdextension.so`
- To disable again, rename back to `.disabled`.
