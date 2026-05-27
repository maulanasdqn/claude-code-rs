use stynx_code_errors::AppResult;

pub trait SleepInhibitor: Send + Sync {
    fn inhibit(&self) -> AppResult<SleepGuard>;
}

pub struct SleepGuard {
    _private: (),
}

impl SleepGuard {
    fn new() -> Self {
        tracing::info!("sleep inhibited");
        Self { _private: () }
    }
}

impl Drop for SleepGuard {
    fn drop(&mut self) {
        tracing::info!("sleep re-enabled");
    }
}

pub struct NoopInhibitor;

impl NoopInhibitor {
    pub fn new() -> Self {
        Self
    }
}

impl SleepInhibitor for NoopInhibitor {
    fn inhibit(&self) -> AppResult<SleepGuard> {
        tracing::warn!("using noop sleep inhibitor; system may still sleep");
        Ok(SleepGuard::new())
    }
}
