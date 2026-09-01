use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Local};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::EnvFilter;

struct Inner {
    dir: PathBuf,
    active_date: String,
    file: Option<File>,
}

impl Inner {
    fn new(dir: PathBuf) -> io::Result<Self> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let log_path = dir.join("orbit.log");

        // If orbit.log already exists, check if it was last modified on a previous day
        if log_path.exists() {
            let modified = std::fs::metadata(&log_path)
                .ok()
                .and_then(|m| m.modified().ok());
            if let Some(modified) = modified {
                let mod_datetime: DateTime<Local> = modified.into();
                let mod_date = mod_datetime.format("%Y-%m-%d").to_string();
                if mod_date != today {
                    // Rename existing orbit.log to orbit.log.<mod_date>
                    let archive_path = dir.join(format!("orbit.log.{}", mod_date));
                    let _ = std::fs::rename(&log_path, &archive_path);
                }
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Self {
            dir,
            active_date: today,
            file: Some(file),
        })
    }

    fn cleanup_old_logs(dir: &std::path::Path, max_logs: usize) {
        if max_logs == 0 {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) => {
                tracing::warn!(dir = ?dir, error = %err, "Failed to read logs directory for cleanup");
                return;
            }
        };

        // Collect all archived log files matching "orbit.log."
        let mut archive_files = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("orbit.log.") {
                        archive_files.push(path);
                    }
                }
            }
        }

        // Sort descending so the newest archives come first
        archive_files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        // Keep at most max_logs total log files (1 active orbit.log + max_logs - 1 archives)
        let max_archives_to_keep = max_logs.saturating_sub(1);

        if archive_files.len() > max_archives_to_keep {
            for old_file in &archive_files[max_archives_to_keep..] {
                if let Err(err) = std::fs::remove_file(old_file) {
                    tracing::warn!(file = ?old_file, error = %err, "Failed to remove old log file during cleanup");
                } else {
                    tracing::debug!(file = ?old_file, "Cleaned up old log file");
                }
            }
        }
    }

    fn cleanup(&self) {
        let max_logs = crate::config::OrbitConfig::load().max_log_files();
        Self::cleanup_old_logs(&self.dir, max_logs);
    }

    fn check_rotate(&mut self) -> io::Result<()> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        if today != self.active_date {
            // Drop current file handle so it can be safely renamed
            self.file = None;

            let log_path = self.dir.join("orbit.log");
            let archive_path = self.dir.join(format!("orbit.log.{}", self.active_date));
            if log_path.exists() {
                let _ = std::fs::rename(&log_path, &archive_path);
            }

            self.active_date = today;
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)?;
            self.file = Some(file);

            self.cleanup();
        }
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.check_rotate()?;
        if let Some(ref mut f) = self.file {
            f.write_all(buf)?;
            f.flush()?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct RotatingFileAppender {
    inner: Arc<Mutex<Inner>>,
}

impl RotatingFileAppender {
    pub fn new(dir: PathBuf) -> Self {
        let inner = Inner::new(dir).expect("Failed to initialize rotating file appender");
        inner.cleanup();
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl Write for RotatingFileAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("Lock poisoned"))?;
        inner.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("Lock poisoned"))?;
        if let Some(ref mut f) = inner.file {
            f.flush()?;
        }
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for RotatingFileAppender {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Initialize tracing with file logging in `~/.orbit/logs/orbit.log` and stdout.
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let logs_dir = crate::config::OrbitConfig::logs_dir()
        .or_else(crate::config::OrbitConfig::config_dir)
        .ok_or("Could not determine Orbit logs directory")?;
    std::fs::create_dir_all(&logs_dir)?;

    let appender = RotatingFileAppender::new(logs_dir.clone());
    let writer = appender.and(io::stdout);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,orbit_engine=info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(writer)
        .with_ansi(false)
        .init();

    let _ = tracing_log::LogTracer::init();

    tracing::info!(
        "Orbit Activity Logger initialized. Active log: {:?}",
        logs_dir.join("orbit.log")
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotating_file_appender_writes() {
        let temp_dir = std::env::temp_dir().join(format!("orbit_test_log_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut appender = RotatingFileAppender::new(temp_dir.clone());
        writeln!(appender, "Test log line 1").unwrap();
        writeln!(appender, "Test log line 2").unwrap();

        let log_file = temp_dir.join("orbit.log");
        assert!(log_file.exists());

        let content = std::fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("Test log line 1"));
        assert!(content.contains("Test log line 2"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_rotating_file_appender_rotates_on_date_change() {
        let temp_dir = std::env::temp_dir().join(format!("orbit_test_log_rotate_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut appender = RotatingFileAppender::new(temp_dir.clone());
        writeln!(appender, "Day 1 content").unwrap();

        // Simulate previous active date
        {
            let mut inner = appender.inner.lock().unwrap();
            inner.active_date = "2026-08-20".to_string();
        }

        // Write new line, triggering rotation
        writeln!(appender, "Day 2 content").unwrap();

        // Check that orbit.log.2026-08-20 was created with Day 1 content
        let rotated_file = temp_dir.join("orbit.log.2026-08-20");
        assert!(rotated_file.exists());
        let rotated_content = std::fs::read_to_string(&rotated_file).unwrap();
        assert!(rotated_content.contains("Day 1 content"));

        // Check that current orbit.log has Day 2 content
        let active_file = temp_dir.join("orbit.log");
        assert!(active_file.exists());
        let active_content = std::fs::read_to_string(&active_file).unwrap();
        assert!(active_content.contains("Day 2 content"));
        assert!(!active_content.contains("Day 1 content"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cleanup_old_logs_retention() {
        let temp_dir = std::env::temp_dir().join(format!("orbit_test_log_cleanup_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create active log file
        std::fs::write(temp_dir.join("orbit.log"), "active").unwrap();

        // Create 5 archived logs with different dates
        let dates = vec![
            "2026-08-01",
            "2026-08-02",
            "2026-08-03",
            "2026-08-04",
            "2026-08-05",
        ];

        for d in &dates {
            std::fs::write(temp_dir.join(format!("orbit.log.{}", d)), format!("log-{}", d)).unwrap();
        }

        // Keep 3 total logs (1 active + 2 newest archives: 2026-08-05 and 2026-08-04)
        Inner::cleanup_old_logs(&temp_dir, 3);

        assert!(temp_dir.join("orbit.log").exists());
        assert!(temp_dir.join("orbit.log.2026-08-05").exists());
        assert!(temp_dir.join("orbit.log.2026-08-04").exists());
        assert!(!temp_dir.join("orbit.log.2026-08-03").exists());
        assert!(!temp_dir.join("orbit.log.2026-08-02").exists());
        assert!(!temp_dir.join("orbit.log.2026-08-01").exists());

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cleanup_old_logs_zero_unlimited() {
        let temp_dir = std::env::temp_dir().join(format!("orbit_test_log_unlimited_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        std::fs::write(temp_dir.join("orbit.log"), "active").unwrap();
        std::fs::write(temp_dir.join("orbit.log.2026-08-01"), "old1").unwrap();
        std::fs::write(temp_dir.join("orbit.log.2026-08-02"), "old2").unwrap();

        // 0 means keep all / unlimited
        Inner::cleanup_old_logs(&temp_dir, 0);

        assert!(temp_dir.join("orbit.log").exists());
        assert!(temp_dir.join("orbit.log.2026-08-01").exists());
        assert!(temp_dir.join("orbit.log.2026-08-02").exists());

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
