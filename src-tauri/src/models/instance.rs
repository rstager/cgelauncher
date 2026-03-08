use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmStatus {
    Running,
    Starting,
    Stopping,
    Stopped,
    NotFound,
}

impl VmStatus {
    /// Map gcloud instance status strings to our domain enum.
    pub fn from_gcloud(status: &str) -> Self {
        match status {
            "RUNNING" => VmStatus::Running,
            "PROVISIONING" | "STAGING" => VmStatus::Starting,
            "STOPPING" | "SUSPENDING" => VmStatus::Stopping,
            "TERMINATED" | "STOPPED" | "SUSPENDED" => VmStatus::Stopped,
            _ => VmStatus::Stopped,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmStatusUpdate {
    pub disk_name: String,
    pub instance_name: String,
    pub status: VmStatus,
    pub machine_type: Option<String>,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u32>,
    pub memory_gb: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_running_status() {
        assert_eq!(VmStatus::from_gcloud("RUNNING"), VmStatus::Running);
    }

    #[test]
    fn maps_provisioning_to_starting() {
        assert_eq!(VmStatus::from_gcloud("PROVISIONING"), VmStatus::Starting);
    }

    #[test]
    fn maps_staging_to_starting() {
        assert_eq!(VmStatus::from_gcloud("STAGING"), VmStatus::Starting);
    }

    #[test]
    fn maps_stopping_status() {
        assert_eq!(VmStatus::from_gcloud("STOPPING"), VmStatus::Stopping);
    }

    #[test]
    fn maps_terminated_to_stopped() {
        assert_eq!(VmStatus::from_gcloud("TERMINATED"), VmStatus::Stopped);
    }

    #[test]
    fn maps_unknown_to_stopped() {
        assert_eq!(VmStatus::from_gcloud("SOMETHING_ELSE"), VmStatus::Stopped);
    }

    #[test]
    fn status_update_serializes() {
        let update = VmStatusUpdate {
            disk_name: "test-disk".into(),
            instance_name: "test-disk-vm".into(),
            status: VmStatus::Running,
            machine_type: Some("n1-standard-8".into()),
            gpu_type: Some("nvidia-tesla-t4".into()),
            gpu_count: Some(4),
            memory_gb: Some(30.0),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("Running"));
        assert!(json.contains("test-disk"));
    }
}
