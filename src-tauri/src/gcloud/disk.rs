use crate::gcloud::executor::{GcloudError, GcloudRunner};
use crate::models::Disk;

pub async fn list_disks(
    runner: &dyn GcloudRunner,
    zone: &str,
) -> Result<Vec<Disk>, GcloudError> {
    let zone_arg = format!("--zones={zone}");
    let output = runner
        .run(&["compute", "disks", "list", &zone_arg])
        .await?;

    crate::models::disk::parse_disks(&output).map_err(|e| GcloudError {
        message: e,
        command: format!("compute disks list --zones={zone}"),
        exit_code: -1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcloud::executor::FakeRunner;

    const SAMPLE_OUTPUT: &str = r#"[
        {
            "name": "disk-a",
            "sizeGb": "100",
            "status": "READY",
            "type": "projects/p/zones/z/diskTypes/pd-ssd",
            "users": ["projects/p/zones/z/instances/disk-a-vm"]
        },
        {
            "name": "disk-b",
            "sizeGb": "200",
            "status": "READY",
            "type": "projects/p/zones/z/diskTypes/pd-balanced"
        }
    ]"#;

    #[tokio::test]
    async fn lists_and_parses_disks() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute disks list", SAMPLE_OUTPUT);

        let disks = list_disks(&runner, "us-central1-a").await.unwrap();
        assert_eq!(disks.len(), 2);
        assert_eq!(disks[0].name, "disk-a");
        assert_eq!(disks[0].attached_to.as_deref(), Some("disk-a-vm"));
        assert_eq!(disks[1].name, "disk-b");
        assert!(disks[1].attached_to.is_none());
    }

    #[tokio::test]
    async fn propagates_gcloud_error() {
        let mut runner = FakeRunner::new();
        runner.on_error("compute disks list", "permission denied", 1);

        let result = list_disks(&runner, "us-central1-a").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("permission denied"));
    }

    #[tokio::test]
    async fn empty_disk_list() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute disks list", "[]");

        let disks = list_disks(&runner, "us-central1-a").await.unwrap();
        assert!(disks.is_empty());
    }
}
