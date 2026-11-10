use std::path::PathBuf;

pub fn open_browser_url(url: &str) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

pub fn pick_kubeconfig_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Kubeconfig / YAML", &["yaml", "yml", "conf", "config", "*"])
        .pick_file()
}

pub fn save_log_file(default_filename: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_file_name(default_filename)
        .add_filter("Log Files", &["log", "txt"])
        .save_file()
}
