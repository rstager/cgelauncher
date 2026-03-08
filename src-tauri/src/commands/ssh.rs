use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct SshConfigResponse {
    pub ssh_host: String,
    pub config_path: String,
}

#[tauri::command]
pub async fn configure_ssh(
    state: State<'_, AppState>,
    instance_name: String,
) -> Result<SshConfigResponse, String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    let project = prefs.project.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    let result =
        crate::gcloud::ssh::configure_ssh(&*runner, &instance_name, &zone, &project)
            .await
            .map_err(|e| e.to_string())?;

    Ok(SshConfigResponse {
        ssh_host: result.ssh_host,
        config_path: result.config_path,
    })
}
