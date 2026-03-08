use crate::gcloud::executor::{GcloudError, GcloudRunner};

pub struct SshConfigResult {
    pub ssh_host: String,
    pub config_path: String,
}

/// Runs `gcloud compute config-ssh` and returns the SSH host alias.
pub async fn configure_ssh(
    runner: &dyn GcloudRunner,
    instance_name: &str,
    zone: &str,
    project: &str,
) -> Result<SshConfigResult, GcloudError> {
    runner
        .run(&["compute", "config-ssh", "--project", project, "--quiet"])
        .await?;

    let ssh_host = format!("{instance_name}.{zone}.{project}");
    let config_path = dirs::home_dir()
        .map(|h| h.join(".ssh").join("config").to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.ssh/config".into());

    Ok(SshConfigResult {
        ssh_host,
        config_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcloud::executor::FakeRunner;

    #[tokio::test]
    async fn produces_correct_ssh_host() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute config-ssh", "");

        let result = configure_ssh(&runner, "my-vm", "us-central1-a", "my-project")
            .await
            .unwrap();
        assert_eq!(result.ssh_host, "my-vm.us-central1-a.my-project");
        assert!(result.config_path.contains(".ssh"));
    }

    #[tokio::test]
    async fn propagates_error() {
        let mut runner = FakeRunner::new();
        runner.on_error("compute config-ssh", "ssh error", 1);

        let result = configure_ssh(&runner, "my-vm", "us-central1-a", "my-project").await;
        assert!(result.is_err());
    }
}
