use crate::models::Disk;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_disks(state: State<'_, AppState>) -> Result<Vec<Disk>, String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    crate::gcloud::disk::list_disks(&*runner, &zone)
        .await
        .map_err(|e| e.to_string())
}
