#[cfg(windows)]
fn embed_windows_resources() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let icon_path = "../src-tauri/icons/icon.ico";
    if !Path::new(icon_path).is_file() {
        return Err(format!("Windows icon resource not found at {icon_path}").into());
    }

    let mut res = tauri_winres::WindowsResource::new();
    res.set_icon_with_id(icon_path, "1");
    res.set("FileDescription", "Orbit");
    res.set("ProductName", "Orbit");
    res.set("OriginalFilename", "orbit.exe");
    res.set("InternalName", "Orbit");
    res.compile_for(&["orbit"])?;
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../src-tauri/icons/icon.ico");

    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
            && let Err(err) = embed_windows_resources()
        {
            eprintln!("Failed to embed Windows resources for orbit binary: {err}");
            std::process::exit(1);
        }
    }
}
