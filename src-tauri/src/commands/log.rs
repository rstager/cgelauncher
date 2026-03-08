use crate::gcloud::executor::{get_command_logs, GcloudCommandLogEntry};

#[tauri::command]
pub async fn get_gcloud_logs() -> Result<Vec<GcloudCommandLogEntry>, String> {
    Ok(get_command_logs())
}
