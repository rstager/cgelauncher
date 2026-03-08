use crate::models::machine::MachineConfig;
use crate::models::pricing::PricingEstimate;
use crate::state::AppState;

#[tauri::command]
pub async fn estimate_pricing(
    state: tauri::State<'_, AppState>,
    config: MachineConfig,
) -> Result<PricingEstimate, String> {
    let cache_guard = state.pricing_cache.lock().await;
    crate::gcloud::pricing::estimate_pricing(&config, cache_guard.as_ref())
}
