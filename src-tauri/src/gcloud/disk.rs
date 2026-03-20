use crate::gcloud::executor::{GcloudError, GcloudRunner};
use crate::models::Disk;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub name: String,
    pub family: Option<String>,
    pub description: Option<String>,
    pub disk_size_gb: Option<String>,
    pub self_link: String,
}

pub async fn list_images(
    runner: &dyn GcloudRunner,
    image_project: &str,
    filter: Option<&str>,
) -> Result<Vec<ImageInfo>, GcloudError> {
    let project_arg = format!("--project={image_project}");
    let mut args = vec!["compute", "images", "list", &project_arg];
    let filter_arg;
    if let Some(f) = filter {
        filter_arg = format!("--filter={f}");
        args.push(&filter_arg);
    }
    let output = runner.run(&args).await?;
    serde_json::from_str::<Vec<ImageInfo>>(&output).map_err(|e| GcloudError {
        message: format!("Failed to parse images response: {e}"),
        command: format!("compute images list --project={image_project}"),
        exit_code: -1,
    })
}

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

pub async fn create_disk(
    runner: &dyn GcloudRunner,
    zone: &str,
    name: &str,
    size_gb: u32,
    disk_type: &str,
    source_image: Option<&str>,
) -> Result<(), GcloudError> {
    let zone_arg = format!("--zone={zone}");
    let size_arg = format!("--size={size_gb}GB");
    let type_arg = format!("--type={disk_type}");
    let mut args = vec!["compute", "disks", "create", name, &zone_arg, &size_arg, &type_arg];
    let image_arg;
    if let Some(image) = source_image {
        image_arg = format!("--image={image}");
        args.push(&image_arg);
    }
    runner.run(&args).await?;
    Ok(())
}

pub async fn delete_disk(
    runner: &dyn GcloudRunner,
    zone: &str,
    name: &str,
) -> Result<(), GcloudError> {
    let zone_arg = format!("--zone={zone}");
    runner
        .run(&["compute", "disks", "delete", name, &zone_arg, "--quiet"])
        .await?;
    Ok(())
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

    #[tokio::test]
    async fn creates_disk_with_correct_args() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute disks create my-disk", "{}");

        let result = create_disk(&runner, "us-central1-a", "my-disk", 100, "pd-ssd", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_disk_propagates_error() {
        let mut runner = FakeRunner::new();
        runner.on_error("compute disks create", "quota exceeded", 1);

        let result = create_disk(&runner, "us-central1-a", "my-disk", 100, "pd-ssd", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("quota exceeded"));
    }

    #[tokio::test]
    async fn creates_disk_with_source_image() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute disks create my-disk", "{}");

        let result = create_disk(
            &runner,
            "us-central1-a",
            "my-disk",
            20,
            "pd-balanced",
            Some("projects/ubuntu-os-cloud/global/images/ubuntu-2204-jammy-v20240101"),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn lists_images_parses_response() {
        let mut runner = FakeRunner::new();
        let json = r#"[{"name":"ubuntu-2204","family":"ubuntu-2204-lts","description":"Ubuntu 22.04","diskSizeGb":"20","selfLink":"projects/ubuntu-os-cloud/global/images/ubuntu-2204"}]"#;
        runner.on_success("compute images list", json);

        let images = list_images(&runner, "ubuntu-os-cloud", None).await.unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].name, "ubuntu-2204");
        assert_eq!(images[0].family.as_deref(), Some("ubuntu-2204-lts"));
        assert_eq!(images[0].disk_size_gb.as_deref(), Some("20"));
    }

    #[tokio::test]
    async fn lists_images_with_filter() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute images list", "[]");

        let images = list_images(&runner, "ubuntu-os-cloud", Some("name:ubuntu-*")).await.unwrap();
        assert!(images.is_empty());
    }

    #[tokio::test]
    async fn deletes_disk_with_correct_args() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute disks delete my-disk", "{}");

        let result = delete_disk(&runner, "us-central1-a", "my-disk").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_disk_propagates_error() {
        let mut runner = FakeRunner::new();
        runner.on_error("compute disks delete", "disk not found", 1);

        let result = delete_disk(&runner, "us-central1-a", "my-disk").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("disk not found"));
    }
}
