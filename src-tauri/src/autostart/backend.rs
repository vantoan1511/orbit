use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutostartEntry {
    pub name: String,
    pub exe_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredEntry {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutostartError {
    HomeDirUnavailable,
    ExecutablePathUnavailable(String),
    UnsafeExecutable(PathBuf),
    Backend {
        code: &'static str,
        detail: String,
    },
}

impl AutostartError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::HomeDirUnavailable => "home_dir_unavailable",
            Self::ExecutablePathUnavailable(_) => "executable_path_unavailable",
            Self::UnsafeExecutable(_) => "unsafe_executable",
            Self::Backend { code, .. } => code,
        }
    }
}

impl std::fmt::Display for AutostartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HomeDirUnavailable => write!(f, "User home directory is unavailable"),
            Self::ExecutablePathUnavailable(err) => {
                write!(f, "Failed to resolve executable path: {err}")
            }
            Self::UnsafeExecutable(path) => {
                write!(f, "Refusing to register test/deps executable: {}", path.display())
            }
            Self::Backend { code, detail } => {
                write!(f, "Autostart backend error [{code}]: {detail}")
            }
        }
    }
}

impl std::error::Error for AutostartError {}

pub trait AutostartBackend: Send + Sync {
    fn is_registered(&self, name: &str) -> Result<bool, AutostartError>;
    fn register(&self, entry: &AutostartEntry) -> Result<(), AutostartError>;
    fn unregister(&self, name: &str) -> Result<(), AutostartError>;
    fn list(&self) -> Result<Vec<RegisteredEntry>, AutostartError>;
}
