
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use log::{info, error};
use simplelog::*;
use clap::Parser;
use std::io;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the downloaded zip file containing the update
    #[arg(short, long)]
    zip_path: String,

    /// Directory where the engine is installed
    #[arg(short, long)]
    target_dir: String,

    /// The name of the executable to restart after update
    #[arg(short, long)]
    executable_name: String,
}

fn main() {
    let args = Args::parse();

    // Initialize logging
    let mut log_path = PathBuf::from(&args.target_dir);
    log_path.push("updater.log");
    
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        fs::File::create(&log_path).unwrap_or_else(|_| fs::File::create("updater.log").unwrap())
    );

    info!("Orbit Updater Started");
    info!("Zip path: {}", args.zip_path);
    info!("Target dir: {}", args.target_dir);
    info!("Executable name: {}", args.executable_name);

    let target_path = Path::new(&args.target_dir);
    let backup_path = target_path.join(".rollback");

    // 1. Create Backup
    info!("Creating backup in {:?}", backup_path);
    if backup_path.exists() {
        if let Err(e) = fs::remove_dir_all(&backup_path) {
            error!("Failed to remove old backup directory: {}", e);
            exit(1);
        }
    }
    
    if let Err(e) = fs::create_dir_all(&backup_path) {
        error!("Failed to create backup directory: {}", e);
        exit(1);
    }

    if let Err(e) = backup_directory(&target_path, &backup_path) {
         error!("Failed to backup directory: {}", e);
         exit(1);
    }

    // 2. Wait a bit for the main engine to fully exit (since the engine spawned us and is exiting)
    std::thread::sleep(std::time::Duration::from_secs(2));

    // 3. Extract Zip and Replace Files
    info!("Extracting zip {:?}", args.zip_path);
    if let Err(e) = extract_and_replace(&args.zip_path, &target_path) {
        error!("Failed to extract and replace: {}", e);
        info!("Initiating rollback...");
        
        if let Err(rollback_err) = restore_backup(&backup_path, &target_path) {
            error!("CRITICAL: Rollback failed! System may be in an inconsistent state. Error: {}", rollback_err);
            exit(1);
        } else {
            info!("Rollback successful.");
            // Restart original engine
            restart_engine(&target_path, &args.executable_name);
            exit(1);
        }
    }

    info!("Update applied successfully.");

    // 4. Re-launch the main engine
    restart_engine(&target_path, &args.executable_name);
    
    // Clean up update zip if needed
    let _ = fs::remove_file(&args.zip_path);
}

fn restart_engine(target_dir: &Path, executable_name: &str) {
    let exe_path = target_dir.join(executable_name);
    info!("Restarting engine at {:?}", exe_path);
    match Command::new(&exe_path)
        .current_dir(target_dir)
        .spawn() {
        Ok(_) => {
            info!("Engine restarted successfully.");
        }
        Err(e) => {
            error!("Failed to restart engine: {}", e);
        }
    }
}

fn backup_directory(src: &Path, backup_dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        
        // Don't backup the backup dir itself or log file
        if path == backup_dir || path.file_name() == Some(std::ffi::OsStr::new("updater.log")) {
            continue;
        }

        let dest = backup_dir.join(entry.file_name());
        if path.is_dir() {
            fs::create_dir_all(&dest)?;
            backup_directory(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

fn restore_backup(backup_dir: &Path, target_dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        let dest = target_dir.join(entry.file_name());
        
        if path.is_dir() {
            if !dest.exists() {
                fs::create_dir_all(&dest)?;
            }
            restore_backup(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

fn extract_and_replace(zip_path: &str, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Unix permissions (if applicable)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
