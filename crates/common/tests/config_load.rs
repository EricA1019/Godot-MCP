// Basic test ensures common crate compiles and config type exists
#[test]
fn types_exist() {
    let _ = godot_mcp_common_types();
}

fn godot_mcp_common_types() {
    use common::{AppConfig, ServerConfig};
    let _cfg = AppConfig { server: ServerConfig { host: "127.0.0.1".into(), port: 8080, auto_start_watchers: true } };
}

//EOF