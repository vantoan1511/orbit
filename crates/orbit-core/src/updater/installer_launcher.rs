use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallerLaunchError {
    ElevationDenied,
    Failed { os_code: Option<u32>, detail: String },
}

pub trait InstallerLauncher: Send + Sync {
    fn launch(&self, installer: &Path) -> Result<(), InstallerLaunchError>;
}

pub struct ShellInstallerLauncher;

#[cfg(windows)]
impl InstallerLauncher for ShellInstallerLauncher {
    fn launch(&self, installer: &Path) -> Result<(), InstallerLaunchError> {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::GetLastError;
        use windows_sys::Win32::UI::Shell::{
            ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let verb_wide: Vec<u16> = std::ffi::OsStr::new("open")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let file_wide: Vec<u16> = installer
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // SAFETY: `info` is allocated on the stack and zero-initialized with `cbSize` set
        // to `size_of::<SHELLEXECUTEINFOW>()`. `verb_wide` and `file_wide` are null-terminated
        // UTF-16 buffers that outlive the synchronous `ShellExecuteExW` call (SEE_MASK_NOASYNC).
        let success = unsafe {
            let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
            info.fMask = SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI;
            info.lpVerb = verb_wide.as_ptr();
            info.lpFile = file_wide.as_ptr();
            info.nShow = SW_SHOWNORMAL;
            ShellExecuteExW(&mut info)
        };

        if success == 0 {
            let os_code = unsafe { GetLastError() };
            Err(classify_launch_error(os_code))
        } else {
            Ok(())
        }
    }
}

#[cfg(not(windows))]
impl InstallerLauncher for ShellInstallerLauncher {
    fn launch(&self, installer: &Path) -> Result<(), InstallerLaunchError> {
        match std::process::Command::new(installer).spawn() {
            Ok(_) => Ok(()),
            Err(e) => Err(InstallerLaunchError::Failed {
                os_code: e.raw_os_error().map(|c| c as u32),
                detail: e.to_string(),
            }),
        }
    }
}

pub const ERROR_ACCESS_DENIED: u32 = 5;
pub const ERROR_CANCELLED: u32 = 1223;
pub const UNKNOWN_SHELL_FAILURE_DETAIL: &str = "Unknown Windows Shell execution failure";

pub const ELEVATION_DENIED_CODES: [u32; 2] = [ERROR_CANCELLED, ERROR_ACCESS_DENIED];

pub fn classify_launch_error(os_code: u32) -> InstallerLaunchError {
    if ELEVATION_DENIED_CODES.contains(&os_code) {
        InstallerLaunchError::ElevationDenied
    } else {
        let detail = if os_code == 0 {
            UNKNOWN_SHELL_FAILURE_DETAIL.to_string()
        } else {
            std::io::Error::from_raw_os_error(os_code as i32).to_string()
        };
        InstallerLaunchError::Failed {
            os_code: Some(os_code),
            detail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_launch_error_uac_cancelled() {
        let err = classify_launch_error(1223);
        assert_eq!(err, InstallerLaunchError::ElevationDenied);
    }

    #[test]
    fn test_classify_launch_error_access_denied() {
        let err = classify_launch_error(5);
        assert_eq!(err, InstallerLaunchError::ElevationDenied);
    }

    #[test]
    fn test_classify_launch_error_elevation_required_is_failed() {
        let err = classify_launch_error(740);
        match err {
            InstallerLaunchError::Failed { os_code, detail } => {
                assert_eq!(os_code, Some(740));
                assert!(!detail.is_empty());
            }
            _ => panic!("Expected Failed variant for 740, got {:?}", err),
        }
    }

    #[test]
    fn test_classify_launch_error_unknown() {
        let err = classify_launch_error(2);
        match err {
            InstallerLaunchError::Failed { os_code, detail } => {
                assert_eq!(os_code, Some(2));
                assert!(!detail.is_empty());
            }
            _ => panic!("Expected Failed variant for 2, got {:?}", err),
        }
    }

    #[test]
    fn test_classify_zero_code_uses_fallback_detail() {
        let err = classify_launch_error(0);
        match err {
            InstallerLaunchError::Failed { os_code, detail } => {
                assert_eq!(os_code, Some(0));
                assert_eq!(detail, UNKNOWN_SHELL_FAILURE_DETAIL);
            }
            _ => panic!("Expected Failed variant for 0, got {:?}", err),
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_launcher_reports_missing_file() {
        let launcher = ShellInstallerLauncher;
        let result = launcher.launch(Path::new("C:/nonexistent/x.exe"));
        match result {
            Err(InstallerLaunchError::Failed { os_code, detail }) => {
                assert!(os_code.is_some());
                assert!(!detail.is_empty());
            }
            other => panic!("Expected Err(Failed), got {:?}", other),
        }
    }
}
