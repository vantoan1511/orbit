pub mod autostart;
pub mod command;
pub mod config;
pub mod dispatcher;
pub mod events;
pub mod kubernetes;
pub mod logger;
pub mod sink;
pub mod ui_state;
pub mod updater;

#[cfg(test)]
mod tests {
    #[test]
    fn test_orbit_core_has_zero_tauri_dependencies() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            !cargo_toml.contains("tauri"),
            "orbit-core Cargo.toml must not contain tauri dependencies!"
        );
    }
}
