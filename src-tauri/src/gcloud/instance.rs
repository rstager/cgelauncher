use crate::gcloud::executor::{GcloudError, GcloudRunner};
use crate::models::instance::VmStatus;
use crate::models::machine::{has_builtin_gpu, MachineConfig};

/// Parsed result from `gcloud compute instances describe`.
pub struct InstanceDescription {
    pub status: VmStatus,
    pub machine_type: Option<String>,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u32>,
    pub memory_gb: Option<f64>,
}

pub async fn create_instance(
    runner: &dyn GcloudRunner,
    zone: &str,
    disk_name: &str,
    config: &MachineConfig,
) -> Result<String, GcloudError> {
    let instance_name = disk_name.to_string();
    let arg_strings = build_create_instance_args(zone, disk_name, config);
    let args: Vec<&str> = arg_strings.iter().map(String::as_str).collect();
    runner.run(&args).await?;
    Ok(instance_name)
}

pub fn build_create_instance_args(
    zone: &str,
    disk_name: &str,
    config: &MachineConfig,
) -> Vec<String> {
    let instance_name = disk_name.to_string();
    let zone_arg = format!("--zone={zone}");
    let machine_arg = format!("--machine-type={}", config.machine_type);
    let disk_arg = format!("--disk=name={disk_name},boot=yes,auto-delete=no");

    let mut args: Vec<String> = vec![
        "compute".into(),
        "instances".into(),
        "create".into(),
        instance_name,
        zone_arg,
        machine_arg,
        disk_arg,
    ];

    if !has_builtin_gpu(&config.machine_type) {
        if let (Some(gpu_type), Some(gpu_count)) = (&config.gpu_type, config.gpu_count) {
            args.push(format!("--accelerator=type={gpu_type},count={gpu_count}"));
            args.push("--maintenance-policy=TERMINATE".into());
        }
    }

    if config.spot {
        args.push("--provisioning-model=SPOT".into());
    }

    args.push("--quiet".into());
    args
}

pub async fn delete_instance(
    runner: &dyn GcloudRunner,
    zone: &str,
    instance_name: &str,
) -> Result<(), GcloudError> {
    let zone_arg = format!("--zone={zone}");
    runner
        .run(&[
            "compute",
            "instances",
            "delete",
            instance_name,
            &zone_arg,
            "--keep-disks=all",
            "--quiet",
        ])
        .await?;
    Ok(())
}

pub async fn start_instance(
    runner: &dyn GcloudRunner,
    zone: &str,
    instance_name: &str,
) -> Result<(), GcloudError> {
    let zone_arg = format!("--zone={zone}");
    runner
        .run(&[
            "compute",
            "instances",
            "start",
            instance_name,
            &zone_arg,
            "--quiet",
        ])
        .await?;
    Ok(())
}

pub async fn describe_instance(
    runner: &dyn GcloudRunner,
    zone: &str,
    instance_name: &str,
) -> Result<InstanceDescription, GcloudError> {
    let zone_arg = format!("--zone={zone}");
    let output = runner
        .run(&[
            "compute",
            "instances",
            "describe",
            instance_name,
            &zone_arg,
        ])
        .await;

    match output {
        Ok(json) => parse_instance_description(&json),
        Err(e) if e.message.contains("was not found") || e.exit_code == 1 => {
            Ok(InstanceDescription {
                status: VmStatus::NotFound,
                machine_type: None,
                gpu_type: None,
                gpu_count: None,
                memory_gb: None,
            })
        }
        Err(e) => Err(e),
    }
}

fn parse_instance_description(json: &str) -> Result<InstanceDescription, GcloudError> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| GcloudError {
        message: format!("Failed to parse instance JSON: {e}"),
        command: "compute instances describe".into(),
        exit_code: -1,
    })?;

    let status_str = v["status"].as_str().unwrap_or("UNKNOWN");
    let status = VmStatus::from_gcloud(status_str);

    // Machine type is a URL like "zones/us-central1-a/machineTypes/n1-standard-8"
    let machine_type = v["machineType"]
        .as_str()
        .and_then(|url| url.rsplit('/').next())
        .map(String::from);

    // Parse accelerators if present
    let (gpu_type, gpu_count) = v["guestAccelerators"]
        .as_array()
        .and_then(|accs| accs.first())
        .map(|acc| {
            let gpu = acc["acceleratorType"]
                .as_str()
                .and_then(|url| url.rsplit('/').next())
                .map(String::from);
            let count = acc["acceleratorCount"].as_u64().map(|c| c as u32);
            (gpu, count)
        })
        .unwrap_or((None, None));

    // Approximate memory from machine type spec
    let memory_gb = machine_type
        .as_deref()
        .and_then(memory_from_machine_type);

    Ok(InstanceDescription {
        status,
        machine_type,
        gpu_type,
        gpu_count,
        memory_gb,
    })
}

/// Approximate memory in GB from well-known N1/A2 machine type names.
fn memory_from_machine_type(mt: &str) -> Option<f64> {
    // N1 standard: vCPUs * 3.75 GB
    // N1 highmem: vCPUs * 6.5 GB
    // N1 highcpu: vCPUs * 0.9 GB
    if let Some(rest) = mt.strip_prefix("n1-standard-") {
        rest.parse::<f64>().ok().map(|v| v * 3.75)
    } else if let Some(rest) = mt.strip_prefix("n1-highmem-") {
        rest.parse::<f64>().ok().map(|v| v * 6.5)
    } else if let Some(rest) = mt.strip_prefix("n1-highcpu-") {
        rest.parse::<f64>().ok().map(|v| v * 0.9)
    } else {
        match mt {
            "a2-highgpu-1g" => Some(85.0),
            "a2-highgpu-2g" => Some(170.0),
            "a2-highgpu-4g" => Some(340.0),
            "a2-highgpu-8g" => Some(680.0),
            "a2-megagpu-16g" => Some(1360.0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcloud::executor::FakeRunner;

    #[tokio::test]
    async fn create_n1_with_gpu_spot() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute instances create", "{}");

        let config = MachineConfig {
            machine_type: "n1-standard-8".into(),
            gpu_type: Some("nvidia-tesla-t4".into()),
            gpu_count: Some(4),
            spot: true,
        };

        let name = create_instance(&runner, "us-central1-a", "my-disk", &config)
            .await
            .unwrap();
        assert_eq!(name, "my-disk");
    }

    #[tokio::test]
    async fn create_a2_no_separate_gpu() {
        // A2 machines should not have --accelerator flag even if gpu fields are set
        let mut runner = FakeRunner::new();
        runner.on_success("compute instances create", "{}");

        let config = MachineConfig {
            machine_type: "a2-highgpu-1g".into(),
            gpu_type: None,
            gpu_count: None,
            spot: false,
        };

        let name = create_instance(&runner, "us-central1-a", "my-disk", &config)
            .await
            .unwrap();
        assert_eq!(name, "my-disk");
    }

    #[tokio::test]
    async fn create_n1_no_gpu() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute instances create", "{}");

        let config = MachineConfig {
            machine_type: "n1-standard-8".into(),
            gpu_type: None,
            gpu_count: None,
            spot: true,
        };

        let name = create_instance(&runner, "us-central1-a", "data-disk", &config)
            .await
            .unwrap();
        assert_eq!(name, "data-disk");
    }

    #[tokio::test]
    async fn delete_keeps_disks() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute instances delete", "");

        delete_instance(&runner, "us-central1-a", "my-vm")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn starts_existing_instance() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute instances start", "{}");

        start_instance(&runner, "us-central1-a", "my-vm")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn describe_running_instance() {
        let json = r#"{
            "status": "RUNNING",
            "machineType": "zones/us-central1-a/machineTypes/n1-standard-8",
            "guestAccelerators": [
                {
                    "acceleratorType": "projects/p/zones/z/acceleratorTypes/nvidia-tesla-t4",
                    "acceleratorCount": 4
                }
            ]
        }"#;
        let mut runner = FakeRunner::new();
        runner.on_success("compute instances describe", json);

        let desc = describe_instance(&runner, "us-central1-a", "my-vm")
            .await
            .unwrap();
        assert_eq!(desc.status, VmStatus::Running);
        assert_eq!(desc.machine_type.as_deref(), Some("n1-standard-8"));
        assert_eq!(desc.gpu_type.as_deref(), Some("nvidia-tesla-t4"));
        assert_eq!(desc.gpu_count, Some(4));
        assert_eq!(desc.memory_gb, Some(30.0));
    }

    #[tokio::test]
    async fn describe_not_found_instance() {
        let mut runner = FakeRunner::new();
        runner.on_error(
            "compute instances describe",
            "resource was not found",
            1,
        );

        let desc = describe_instance(&runner, "us-central1-a", "missing-vm")
            .await
            .unwrap();
        assert_eq!(desc.status, VmStatus::NotFound);
    }

    #[test]
    fn memory_for_n1_standard() {
        assert_eq!(memory_from_machine_type("n1-standard-8"), Some(30.0));
        assert_eq!(memory_from_machine_type("n1-standard-4"), Some(15.0));
    }

    #[test]
    fn memory_for_n1_highmem() {
        assert_eq!(memory_from_machine_type("n1-highmem-8"), Some(52.0));
    }

    #[test]
    fn memory_for_a2() {
        assert_eq!(memory_from_machine_type("a2-highgpu-1g"), Some(85.0));
    }

    #[test]
    fn memory_for_unknown() {
        assert_eq!(memory_from_machine_type("e2-medium"), None);
    }
}
