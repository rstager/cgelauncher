pub mod commands;
pub mod gcloud;
pub mod models;
pub mod monitor;
pub mod state;

use gcloud::executor::CliRunner;
use gcloud::pricing_fetch;
use state::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let preferences = commands::config::load_preferences();
    let runner = Arc::new(CliRunner::new(
        preferences.project.clone(),
        preferences.service_account_key_path.clone(),
    ));
    // Load cached pricing from disk (instant, no network)
    let initial_pricing = pricing_fetch::load_cache();
    let app_state = AppState::new(runner, preferences, initial_pricing);

    // Refresh pricing in background (non-blocking)
    let pricing_cache_bg = app_state.pricing_cache.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            match pricing_fetch::fetch_spot_pricing().await {
                Ok(fresh) => {
                    if let Err(e) = pricing_fetch::save_cache(&fresh) {
                        eprintln!("Warning: failed to save pricing cache: {e}");
                    }
                    let mut guard = pricing_cache_bg.lock().await;
                    *guard = Some(fresh);
                }
                Err(e) => {
                    eprintln!("Warning: failed to fetch spot pricing: {e}");
                }
            }
        });
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::disk::list_disks,
            commands::vm::start_vm,
            commands::vm::stop_vm,
            commands::pricing::estimate_pricing,
            commands::auth::check_auth,
            commands::auth::set_service_account,
            commands::ssh::configure_ssh,
            commands::config::get_preferences,
            commands::config::set_preferences,
            commands::config::save_disk_config,
            commands::config::get_disk_config,
            commands::config::save_custom_preset,
            commands::config::delete_custom_preset,
            commands::log::get_gcloud_logs,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
