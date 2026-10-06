fn main() {
    // Compiles only under --features gui; the headless `dsw` CLI build
    // (--no-default-features --bin dsw) doesn't link tauri-build.
    #[cfg(feature = "gui")]
    tauri_build::build()
}
