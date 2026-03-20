use crate::gcloud::instance;
use crate::models::instance::{VmStatus, VmStatusUpdate};
use crate::models::machine::MachineConfig;
use crate::monitor::spawn_monitor;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    disk_name: String,
    config: MachineConfig,
) -> Result<VmStatusUpdate, String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    let project = prefs.project.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    let instance_name = disk_name.clone();

    let (status, machine_type, gpu_type, gpu_count, memory_gb) =
        match instance::describe_instance(&*runner, &zone, &instance_name)
            .await
            .map_err(|e| e.to_string())?
            .status
        {
            VmStatus::NotFound => {
                instance::create_instance(&*runner, &zone, &disk_name, &config)
                    .await
                    .map_err(|e| e.to_string())?;
                (
                    VmStatus::Starting,
                    Some(config.machine_type.clone()),
                    config.gpu_type.clone(),
                    config.gpu_count,
                    None,
                )
            }
            VmStatus::Stopped => {
                instance::start_instance(&*runner, &zone, &instance_name)
                    .await
                    .map_err(|e| e.to_string())?;
                (
                    VmStatus::Starting,
                    Some(config.machine_type.clone()),
                    config.gpu_type.clone(),
                    config.gpu_count,
                    None,
                )
            }
            VmStatus::Running => (
                VmStatus::Running,
                Some(config.machine_type.clone()),
                config.gpu_type.clone(),
                config.gpu_count,
                None,
            ),
            VmStatus::Starting => (
                VmStatus::Starting,
                Some(config.machine_type.clone()),
                config.gpu_type.clone(),
                config.gpu_count,
                None,
            ),
            VmStatus::Stopping => (
                VmStatus::Stopping,
                Some(config.machine_type.clone()),
                config.gpu_type.clone(),
                config.gpu_count,
                None,
            ),
        };

    // Start background status monitor
    let monitor_handle = spawn_monitor(
        app.clone(),
        state.runner.clone(),
        zone.clone(),
        disk_name.clone(),
        instance_name.clone(),
        project.clone(),
    );
    state
        .register_monitor(instance_name.clone(), monitor_handle)
        .await;

    Ok(VmStatusUpdate {
        disk_name,
        instance_name,
        status,
        machine_type,
        gpu_type,
        gpu_count,
        memory_gb,
        external_ip: None,
    })
}

#[tauri::command]
pub async fn stop_vm(
    state: State<'_, AppState>,
    instance_name: String,
) -> Result<bool, String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    instance::delete_instance(&*runner, &zone, &instance_name)
        .await
        .map_err(|e| e.to_string())?;

    // Leave the monitor running — it will poll until the instance is gone and
    // emit NotFound, which is what drives the frontend back to the stopped state.
    Ok(true)
}
