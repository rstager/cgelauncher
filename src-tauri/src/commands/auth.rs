use crate::gcloud::executor::CliRunner;
use crate::models::config::AuthStatus;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn check_auth(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    crate::gcloud::auth::check_auth(&*runner)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_service_account(
    state: State<'_, AppState>,
    key_path: String,
) -> Result<AuthStatus, String> {
    // Validate the key file exists
    if !std::path::Path::new(&key_path).exists() {
        return Err(format!("Service account key file not found: {key_path}"));
    }

    let prefs = state.preferences.lock().await;
    let project = prefs.project.clone();
    drop(prefs);

    let new_runner = Arc::new(CliRunner::new(project, Some(key_path.clone())));
    state.set_runner(new_runner.clone()).await;

    // Update preferences with the key path
    let mut prefs = state.preferences.lock().await;
    prefs.service_account_key_path = Some(key_path);
    drop(prefs);

    // Verify the new credentials work
    crate::gcloud::auth::check_auth(&*new_runner)
        .await
        .map(|mut status| {
            status.method = "service-account".into();
            status
        })
        .map_err(|e| e.to_string())
}
