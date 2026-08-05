use std::env;
use std::fs;

fn main() {
    // Only run on Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        // Read metadata from neutralino.config.json
        let config_str = fs::read_to_string("../../neutralino.config.json")
            .expect("Failed to read neutralino.config.json");
        let config: serde_json::Value = serde_json::from_str(&config_str)
            .expect("Failed to parse neutralino.config.json");

        let mut res = winres::WindowsResource::new();
        // Set the application icon
        res.set_icon("../../public/favicon.ico");

        // Populate metadata from config
        if let Some(name) = config["applicationName"].as_str() {
            res.set("ProductName", name);
            res.set("FileDescription", name);
        }
        if let Some(version) = config["version"].as_str() {
            res.set("ProductVersion", version);
            res.set("FileVersion", version);
        }
        if let Some(author) = config["author"].as_str() {
            res.set("CompanyName", author);
        }
        if let Some(copyright) = config["copyright"].as_str() {
            res.set("LegalCopyright", copyright);
        }

        // Ensure compilation
        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resources: {}", e);
        }

        // Re-run if these files change
        println!("cargo:rerun-if-changed=../../neutralino.config.json");
        println!("cargo:rerun-if-changed=../../public/favicon.ico");
    }
}
