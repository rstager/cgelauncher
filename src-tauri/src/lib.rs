pub mod commands;
pub mod gcloud;
pub mod models;
pub mod monitor;
pub mod oauth;
pub mod state;

use gcloud::executor::{build_runner_from_preferences, ApiRunner};
use gcloud::pricing_fetch;
use state::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let preferences = commands::config::load_preferences();
    let runner = build_runner_from_preferences(&preferences);
    // Load cached pricing from disk (instant, no network)
    let initial_pricing = pricing_fetch::load_cache();
    let app_state = AppState::new(runner, preferences, initial_pricing);

    // Refresh OAuth access token in background when it nears expiry
    let oauth_state_bg = app_state.preferences.clone();
    let oauth_runner_bg = app_state.runner.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await;

                let prefs = oauth_state_bg.lock().await;
                let refresh_token = prefs.oauth_refresh_token.clone();
                let access_token = prefs.api_access_token.clone();
                drop(prefs);

                let Some(refresh_token) = refresh_token else { continue };
                let Some(_access_token) = access_token else { continue };

                let client = reqwest::Client::new();
                match oauth::flow::refresh_access_token(&client, &refresh_token).await {
                    Ok(tokens) => {
                        let mut prefs = oauth_state_bg.lock().await;
                        prefs.api_access_token = Some(tokens.access_token.clone());
                        let prefs_clone = prefs.clone();
                        drop(prefs);

                        let new_runner = Arc::new(ApiRunner::new_with_token(
                            prefs_clone.project.clone(),
                            tokens.access_token,
                        ));
                        let mut runner_guard = oauth_runner_bg.lock().await;
                        *runner_guard = new_runner;
                        drop(runner_guard);

                        let _ = commands::config::persist_preferences_pub(&prefs_clone);
                    }
                    Err(e) => {
                        eprintln!("Warning: OAuth token refresh failed: {e}");
                    }
                }
            }
        });
    });

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
            commands::auth::start_oauth_login,
            commands::auth::revoke_oauth,
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
