use crate::gcloud::executor::GcloudRunner;
use crate::gcloud::pricing_fetch::SpotPricingCache;
use crate::models::UserPreferences;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub struct AppState {
    pub runner: Arc<Mutex<Arc<dyn GcloudRunner>>>,
    pub preferences: Arc<Mutex<UserPreferences>>,
    /// Active monitor tasks keyed by instance name; cancellable via abort().
    pub monitors: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    /// Cached spot pricing from Google's pricing page.
    pub pricing_cache: Arc<Mutex<Option<SpotPricingCache>>>,
}

impl AppState {
    pub fn new(
        runner: Arc<dyn GcloudRunner>,
        preferences: UserPreferences,
        initial_pricing: Option<SpotPricingCache>,
    ) -> Self {
        Self {
            runner: Arc::new(Mutex::new(runner)),
            preferences: Arc::new(Mutex::new(preferences)),
            monitors: Arc::new(Mutex::new(HashMap::new())),
            pricing_cache: Arc::new(Mutex::new(initial_pricing)),
        }
    }

    /// Replace the active runner (e.g. when project or credentials change).
    pub async fn set_runner(&self, runner: Arc<dyn GcloudRunner>) {
        let mut guard = self.runner.lock().await;
        *guard = runner;
    }

    /// Cancel and remove monitor for the given instance.
    pub async fn cancel_monitor(&self, instance_name: &str) {
        let mut monitors = self.monitors.lock().await;
        if let Some(handle) = monitors.remove(instance_name) {
            handle.abort();
        }
    }

    /// Returns true if a monitor is already running for the given instance.
    pub async fn has_monitor(&self, instance_name: &str) -> bool {
        let monitors = self.monitors.lock().await;
        monitors.contains_key(instance_name)
    }

    /// Register a monitor handle for an instance.
    pub async fn register_monitor(&self, instance_name: String, handle: JoinHandle<()>) {
        let mut monitors = self.monitors.lock().await;
        // Cancel any existing monitor for this instance
        if let Some(old) = monitors.insert(instance_name, handle) {
            old.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcloud::executor::FakeRunner;

    #[tokio::test]
    async fn creates_with_defaults() {
        let runner = Arc::new(FakeRunner::new()) as Arc<dyn GcloudRunner>;
        let state = AppState::new(runner, UserPreferences::default(), None);
        let monitors = state.monitors.lock().await;
        assert!(monitors.is_empty());
    }

    #[tokio::test]
    async fn cancel_nonexistent_monitor_is_safe() {
        let runner = Arc::new(FakeRunner::new()) as Arc<dyn GcloudRunner>;
        let state = AppState::new(runner, UserPreferences::default(), None);
        state.cancel_monitor("nonexistent").await;
    }

    #[tokio::test]
    async fn register_and_cancel_monitor() {
        let runner = Arc::new(FakeRunner::new()) as Arc<dyn GcloudRunner>;
        let state = AppState::new(runner, UserPreferences::default(), None);

        let handle = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        });

        state
            .register_monitor("test-vm".into(), handle)
            .await;

        {
            let monitors = state.monitors.lock().await;
            assert!(monitors.contains_key("test-vm"));
        }

        state.cancel_monitor("test-vm").await;

        let monitors = state.monitors.lock().await;
        assert!(!monitors.contains_key("test-vm"));
    }
}
