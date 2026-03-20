use crate::gcloud::disk::ImageInfo;
use crate::models::Disk;
use crate::monitor::spawn_monitor;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn list_disks(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<Disk>, String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    let project = prefs.project.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    let disks = crate::gcloud::disk::list_disks(&*runner, &zone)
        .await
        .map_err(|e| e.to_string())?;

    // Spawn monitors for any VMs already running when the app loads.
    for disk in &disks {
        if let Some(instance_name) = &disk.attached_to {
            if !state.has_monitor(instance_name).await {
                let handle = spawn_monitor(
                    app.clone(),
                    state.runner.clone(),
                    zone.clone(),
                    disk.name.clone(),
                    instance_name.clone(),
                    project.clone(),
                );
                state.register_monitor(instance_name.clone(), handle).await;
            }
        }
    }

    Ok(disks)
}

#[tauri::command]
pub async fn create_disk(
    state: State<'_, AppState>,
    disk_name: String,
    size_gb: u32,
    disk_type: String,
    source_image: Option<String>,
) -> Result<(), String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    crate::gcloud::disk::create_disk(
        &*runner,
        &zone,
        &disk_name,
        size_gb,
        &disk_type,
        source_image.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_images(
    state: State<'_, AppState>,
    image_project: String,
    filter: Option<String>,
) -> Result<Vec<ImageInfo>, String> {
    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    crate::gcloud::disk::list_images(&*runner, &image_project, filter.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_disk(
    state: State<'_, AppState>,
    disk_name: String,
) -> Result<(), String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    crate::gcloud::disk::delete_disk(&*runner, &zone, &disk_name)
        .await
        .map_err(|e| e.to_string())
}
