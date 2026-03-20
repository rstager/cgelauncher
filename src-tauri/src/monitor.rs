use crate::gcloud::executor::GcloudRunner;
use crate::gcloud::instance::describe_instance;
use crate::gcloud::ssh;
use crate::models::instance::{VmStatus, VmStatusUpdate};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Spawn a background task that polls VM status and emits Tauri events.
/// Returns a JoinHandle that can be aborted to stop monitoring.
pub fn spawn_monitor(
    app: AppHandle,
    runner: Arc<Mutex<Arc<dyn GcloudRunner>>>,
    zone: String,
    disk_name: String,
    instance_name: String,
    project: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut last_status: Option<VmStatus> = None;

        loop {
            let runner_guard = runner.lock().await;
            let runner_ref = runner_guard.clone();
            drop(runner_guard);

            let result = describe_instance(&*runner_ref, &zone, &instance_name).await;

            match result {
                Ok(desc) => {
                    let changed = last_status.as_ref() != Some(&desc.status);

                    if changed {
                        // Run config-ssh when VM transitions to Running
                        if desc.status == VmStatus::Running {
                            let ssh_runner = runner_ref.clone();
                            let ssh_instance = instance_name.clone();
                            let ssh_zone = zone.clone();
                            let ssh_project = project.clone();
                            let external_ip = desc.external_ip.clone();
                            tokio::spawn(async move {
                                if let Err(e) = ssh::configure_ssh(&*ssh_runner, &ssh_instance, &ssh_zone, &ssh_project, external_ip).await {
                                    eprintln!("config-ssh failed: {e}");
                                }
                            });
                        }

                        let update = VmStatusUpdate {
                            disk_name: disk_name.clone(),
                            instance_name: instance_name.clone(),
                            status: desc.status.clone(),
                            machine_type: desc.machine_type,
                            gpu_type: desc.gpu_type,
                            gpu_count: desc.gpu_count,
                            memory_gb: desc.memory_gb,
                            external_ip: desc.external_ip.clone(),
                        };

                        // Best-effort emit; ignore errors if frontend is not listening
                        let _ = app.emit("vm-status-update", &update);
                        last_status = Some(desc.status.clone());
                    }

                    // Stop polling when the VM is gone
                    if desc.status == VmStatus::NotFound {
                        break;
                    }
                }
                Err(_) => {
                    // Transient error: retain last known status, retry next interval
                }
            }

            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_interval_is_five_seconds() {
        assert_eq!(POLL_INTERVAL, Duration::from_secs(5));
    }
}
