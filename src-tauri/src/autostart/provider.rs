use std::sync::{Arc, RwLock};
#[cfg(test)]
use std::sync::Mutex;

use crate::autostart::backend::AutostartError;
use crate::autostart::service::AutostartService;

static GLOBAL_SERVICE: RwLock<Option<Arc<AutostartService>>> = RwLock::new(None);

#[cfg(test)]
pub static AUTOSTART_TEST_LOCK: Mutex<()> = Mutex::new(());

pub fn set_global_service(service: Arc<AutostartService>) {
    let mut writer = GLOBAL_SERVICE.write().unwrap_or_else(|e| e.into_inner());
    *writer = Some(service);
}

pub fn service() -> Result<Arc<AutostartService>, AutostartError> {
    {
        let reader = GLOBAL_SERVICE.read().unwrap_or_else(|e| e.into_inner());
        if let Some(ref svc) = *reader {
            return Ok(svc.clone());
        }
    }

    let mut writer = GLOBAL_SERVICE.write().unwrap_or_else(|e| e.into_inner());
    if let Some(ref svc) = *writer {
        return Ok(svc.clone());
    }

    #[cfg(test)]
    {
        let default_test_service = Arc::new(AutostartService::new(
            Arc::new(crate::autostart::memory::InMemoryBackend::new()),
            crate::autostart::service::RunMode::Prod,
            "OrbitTest".to_string(),
            std::path::PathBuf::from("C:\\fake\\orbit.exe"),
        ));
        *writer = Some(default_test_service.clone());
        Ok(default_test_service)
    }

    #[cfg(not(test))]
    {
        let env_service = Arc::new(AutostartService::from_environment()?);
        *writer = Some(env_service.clone());
        Ok(env_service)
    }
}

#[cfg(test)]
pub struct AutostartTestGuard {
    _lock_guard: std::sync::MutexGuard<'static, ()>,
    previous: Option<Arc<AutostartService>>,
}

#[cfg(test)]
impl AutostartTestGuard {
    pub fn install(service: Arc<AutostartService>) -> Self {
        let lock_guard = AUTOSTART_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let previous = {
            let mut writer = GLOBAL_SERVICE.write().unwrap_or_else(|e| e.into_inner());
            let prev = writer.clone();
            *writer = Some(service);
            prev
        };
        Self {
            _lock_guard: lock_guard,
            previous,
        }
    }
}

#[cfg(test)]
impl Drop for AutostartTestGuard {
    fn drop(&mut self) {
        let mut writer = GLOBAL_SERVICE.write().unwrap_or_else(|e| e.into_inner());
        *writer = self.previous.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autostart::backend::AutostartBackend;
    use crate::autostart::service::RunMode;
    use std::path::PathBuf;

    #[test]
    fn cfg_test_default_service_uses_in_memory_backend() {
        let _guard = AUTOSTART_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Reset global service to test default
        {
            let mut writer = GLOBAL_SERVICE.write().unwrap_or_else(|e| e.into_inner());
            *writer = None;
        }

        #[cfg(target_os = "windows")]
        let prod_before = crate::autostart::windows::WindowsRunKeyBackend::production()
            .list()
            .unwrap();

        let svc = service().unwrap();
        svc.set_enabled(true).unwrap();
        assert!(svc.is_enabled().unwrap());

        #[cfg(target_os = "windows")]
        {
            let prod_after = crate::autostart::windows::WindowsRunKeyBackend::production()
                .list()
                .unwrap();
            assert_eq!(prod_before, prod_after);
        }
    }

    #[test]
    fn test_guard_restores_previous_service_on_drop() {
        let custom_svc = Arc::new(AutostartService::new(
            Arc::new(crate::autostart::memory::InMemoryBackend::new()),
            RunMode::Prod,
            "Custom1".to_string(),
            PathBuf::from("C:\\fake\\1.exe"),
        ));

        let initial_name = service().unwrap().app_name().to_string();

        {
            let _guard = AutostartTestGuard::install(custom_svc.clone());
            assert_eq!(service().unwrap().app_name(), "Custom1");
        }

        // After guard drops, previous service is restored
        assert_eq!(service().unwrap().app_name(), initial_name);
    }

    #[test]
    fn from_environment_in_dev_mode_marks_run_mode_dev() {
        let original_dev = std::env::var("ORBIT_DEV").ok();

        unsafe {
            std::env::set_var("ORBIT_DEV", "1");
        }
        let dev_svc = AutostartService::from_environment().unwrap();
        assert_eq!(dev_svc.mode(), RunMode::Dev);

        unsafe {
            std::env::set_var("ORBIT_DEV", "0");
        }
        let prod_svc = AutostartService::from_environment().unwrap();
        assert_eq!(prod_svc.mode(), RunMode::Prod);

        unsafe {
            match original_dev {
                Some(val) => std::env::set_var("ORBIT_DEV", val),
                None => std::env::remove_var("ORBIT_DEV"),
            }
        }
    }
}
