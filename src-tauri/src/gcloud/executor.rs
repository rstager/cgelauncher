use async_trait::async_trait;
use crate::models::UserPreferences;
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::process::Command;

const MAX_LOG_ENTRIES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GcloudCommandLogEntry {
    pub command: String,
    pub response: String,
    pub success: bool,
    pub exit_code: i32,
}

fn command_log_store() -> &'static Mutex<Vec<GcloudCommandLogEntry>> {
    static COMMAND_LOGS: OnceLock<Mutex<Vec<GcloudCommandLogEntry>>> = OnceLock::new();
    COMMAND_LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn push_command_log(entry: GcloudCommandLogEntry) {
    let mut logs = command_log_store()
        .lock()
        .expect("gcloud command log mutex poisoned");
    logs.push(entry);
    if logs.len() > MAX_LOG_ENTRIES {
        let overflow = logs.len() - MAX_LOG_ENTRIES;
        logs.drain(0..overflow);
    }
}

pub fn get_command_logs() -> Vec<GcloudCommandLogEntry> {
    command_log_store()
        .lock()
        .expect("gcloud command log mutex poisoned")
        .clone()
}

pub fn record_command_log(
    command: String,
    response: String,
    success: bool,
    exit_code: i32,
) {
    push_command_log(GcloudCommandLogEntry {
        command,
        response,
        success,
        exit_code,
    });
}

#[cfg(test)]
fn clear_command_logs() {
    command_log_store()
        .lock()
        .expect("gcloud command log mutex poisoned")
        .clear();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcloudError {
    pub message: String,
    pub command: String,
    pub exit_code: i32,
}

impl std::fmt::Display for GcloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "gcloud error (exit {}): {} [cmd: {}]",
            self.exit_code, self.message, self.command
        )
    }
}

impl std::error::Error for GcloudError {}

#[async_trait]
pub trait GcloudRunner: Send + Sync {
    async fn run(&self, args: &[&str]) -> Result<String, GcloudError>;
}

/// Executes gcloud commands as real subprocesses.
pub struct CliRunner {
    pub project: String,
    /// Optional path to a service account JSON key file.
    pub credential_file: Option<String>,
}

impl CliRunner {
    pub fn new(project: String, credential_file: Option<String>) -> Self {
        Self {
            project,
            credential_file,
        }
    }

    /// Build the full argument list including --project and --format=json.
    fn build_args<'a>(&'a self, args: &[&'a str]) -> Vec<&'a str> {
        let mut full_args: Vec<&str> = args.to_vec();
        // Only add --project for commands that accept it (not auth commands)
        let needs_project = !args.first().is_some_and(|&a| a == "auth");
        if needs_project && !self.project.is_empty() {
            full_args.push("--project");
            full_args.push(&self.project);
        }
        // Add --format=json for commands that produce parseable output
        let needs_format = args
            .first()
            .is_some_and(|&a| a == "compute" || a == "auth");
        if needs_format {
            full_args.push("--format=json");
        }
        full_args
    }
}

/// Executes Google Compute API requests directly (without requiring gcloud installed).
pub struct ApiRunner {
    pub project: String,
    pub access_token: Option<String>,
    client: Client,
}

impl ApiRunner {
    pub fn new(project: String, access_token: Option<String>) -> Self {
        Self {
            project,
            access_token,
            client: Client::new(),
        }
    }

    fn required_token(&self, command: &str) -> Result<String, GcloudError> {
        self.access_token
            .clone()
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| GcloudError {
                message: "API mode requires an access token".into(),
                command: command.into(),
                exit_code: 401,
            })
    }

    fn required_project(&self, command: &str) -> Result<String, GcloudError> {
        if self.project.trim().is_empty() {
            return Err(GcloudError {
                message: "project is required in API mode".into(),
                command: command.into(),
                exit_code: 400,
            });
        }
        Ok(self.project.clone())
    }

    fn get_flag_value(args: &[&str], prefix: &str) -> Option<String> {
        args.iter()
            .find_map(|arg| arg.strip_prefix(prefix).map(|v| v.to_string()))
    }

    async fn request(
        &self,
        method: Method,
        command: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<String, GcloudError> {
        let token = self.required_token(command)?;
        let url = format!("https://compute.googleapis.com/compute/v1{path}");
        let mut request = self
            .client
            .request(method, &url)
            .bearer_auth(token)
            .header("Accept", "application/json");

        if let Some(payload) = body {
            request = request.json(&payload);
        }

        let response = request.send().await.map_err(|e| GcloudError {
            message: format!("API request failed: {e}"),
            command: command.to_string(),
            exit_code: -1,
        })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if status.is_success() {
            Ok(text)
        } else {
            Err(GcloudError {
                message: if text.is_empty() {
                    status
                        .canonical_reason()
                        .unwrap_or("request failed")
                        .to_string()
                } else {
                    text
                },
                command: command.to_string(),
                exit_code: status.as_u16() as i32,
            })
        }
    }

    async fn run_api(&self, args: &[&str]) -> Result<String, GcloudError> {
        let cmd = format!("gcloud {}", args.join(" "));

        if args.starts_with(&["auth", "print-access-token"]) {
            return self.required_token(&cmd);
        }

        if args.starts_with(&["auth", "list"]) {
            self.required_token(&cmd)?;
            return Ok("[{\"account\":\"api-token\",\"status\":\"ACTIVE\"}]".into());
        }

        if args.len() >= 3
            && args[0] == "compute"
            && args[1] == "disks"
            && args[2] == "list"
        {
            let project = self.required_project(&cmd)?;
            let zone = Self::get_flag_value(args, "--zones=")
                .or_else(|| Self::get_flag_value(args, "--zone="))
                .ok_or_else(|| GcloudError {
                message: "missing --zones flag".into(),
                command: cmd.clone(),
                exit_code: 400,
            })?;
            let raw = self
                .request(
                    Method::GET,
                    &cmd,
                    &format!("/projects/{project}/zones/{zone}/disks"),
                    None,
                )
                .await?;
            let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| GcloudError {
                message: format!("Failed to parse disks response: {e}"),
                command: cmd,
                exit_code: -1,
            })?;
            let items = value
                .get("items")
                .cloned()
                .unwrap_or_else(|| serde_json::Value::Array(vec![]));
            return serde_json::to_string(&items).map_err(|e| GcloudError {
                message: format!("Failed to encode disks list: {e}"),
                command: "compute disks list".into(),
                exit_code: -1,
            });
        }

        if args.len() >= 4
            && args[0] == "compute"
            && args[1] == "instances"
            && args[2] == "create"
        {
            let project = self.required_project(&cmd)?;
            let zone = Self::get_flag_value(args, "--zone=").ok_or_else(|| GcloudError {
                message: "missing --zone flag".into(),
                command: cmd.clone(),
                exit_code: 400,
            })?;
            let instance_name = args[3].to_string();
            let machine_type = Self::get_flag_value(args, "--machine-type=").ok_or_else(|| {
                GcloudError {
                    message: "missing --machine-type flag".into(),
                    command: cmd.clone(),
                    exit_code: 400,
                }
            })?;
            let disk_flag = Self::get_flag_value(args, "--disk=").ok_or_else(|| GcloudError {
                message: "missing --disk flag".into(),
                command: cmd.clone(),
                exit_code: 400,
            })?;
            let disk_name = disk_flag
                .split(',')
                .find_map(|part| part.strip_prefix("name="))
                .ok_or_else(|| GcloudError {
                    message: "disk flag missing disk name".into(),
                    command: cmd.clone(),
                    exit_code: 400,
                })?
                .to_string();

            let mut body = json!({
                "name": instance_name,
                "machineType": format!("zones/{zone}/machineTypes/{machine_type}"),
                "disks": [{
                    "boot": true,
                    "autoDelete": false,
                    "source": format!("projects/{project}/zones/{zone}/disks/{disk_name}")
                }]
            });

            if let Some(accelerator) = Self::get_flag_value(args, "--accelerator=") {
                let mut gpu_type: Option<String> = None;
                let mut gpu_count: Option<u32> = None;
                for part in accelerator.split(',') {
                    if let Some(value) = part.strip_prefix("type=") {
                        gpu_type = Some(value.to_string());
                    }
                    if let Some(value) = part.strip_prefix("count=") {
                        gpu_count = value.parse::<u32>().ok();
                    }
                }

                if let (Some(gpu_type), Some(gpu_count)) = (gpu_type, gpu_count) {
                    body["guestAccelerators"] = json!([{ 
                        "acceleratorType": format!("projects/{project}/zones/{zone}/acceleratorTypes/{gpu_type}"),
                        "acceleratorCount": gpu_count
                    }]);
                }
            }

            if args.contains(&"--provisioning-model=SPOT") {
                body["scheduling"] = json!({
                    "provisioningModel": "SPOT",
                    "automaticRestart": false,
                    "onHostMaintenance": "TERMINATE"
                });
            }

            return self
                .request(
                    Method::POST,
                    &cmd,
                    &format!("/projects/{project}/zones/{zone}/instances"),
                    Some(body),
                )
                .await;
        }

        if args.len() >= 4
            && args[0] == "compute"
            && args[1] == "instances"
            && args[2] == "start"
        {
            let project = self.required_project(&cmd)?;
            let zone = Self::get_flag_value(args, "--zone=").ok_or_else(|| GcloudError {
                message: "missing --zone flag".into(),
                command: cmd.clone(),
                exit_code: 400,
            })?;
            let instance_name = args[3];
            return self
                .request(
                    Method::POST,
                    &cmd,
                    &format!(
                        "/projects/{project}/zones/{zone}/instances/{instance_name}/start"
                    ),
                    Some(json!({})),
                )
                .await;
        }

        if args.len() >= 4
            && args[0] == "compute"
            && args[1] == "instances"
            && args[2] == "delete"
        {
            let project = self.required_project(&cmd)?;
            let zone = Self::get_flag_value(args, "--zone=").ok_or_else(|| GcloudError {
                message: "missing --zone flag".into(),
                command: cmd.clone(),
                exit_code: 400,
            })?;
            let instance_name = args[3];
            return self
                .request(
                    Method::DELETE,
                    &cmd,
                    &format!(
                        "/projects/{project}/zones/{zone}/instances/{instance_name}"
                    ),
                    None,
                )
                .await;
        }

        if args.len() >= 4
            && args[0] == "compute"
            && args[1] == "instances"
            && args[2] == "describe"
        {
            let project = self.required_project(&cmd)?;
            let zone = Self::get_flag_value(args, "--zone=").ok_or_else(|| GcloudError {
                message: "missing --zone flag".into(),
                command: cmd.clone(),
                exit_code: 400,
            })?;
            let instance_name = args[3];
            let result = self
                .request(
                    Method::GET,
                    &cmd,
                    &format!(
                        "/projects/{project}/zones/{zone}/instances/{instance_name}"
                    ),
                    None,
                )
                .await;

            if let Err(err) = &result {
                if err.exit_code == StatusCode::NOT_FOUND.as_u16() as i32 {
                    return Err(GcloudError {
                        message: format!("The resource '{instance_name}' was not found"),
                        command: err.command.clone(),
                        exit_code: 1,
                    });
                }
            }

            return result;
        }

        if args.starts_with(&["compute", "config-ssh"]) {
            self.required_token(&cmd)?;
            return Ok("API mode: skipping gcloud config-ssh".into());
        }

        Err(GcloudError {
            message: format!("Unsupported command in API mode: {}", args.join(" ")),
            command: cmd,
            exit_code: 400,
        })
    }
}

#[async_trait]
impl GcloudRunner for ApiRunner {
    async fn run(&self, args: &[&str]) -> Result<String, GcloudError> {
        let cmd = format!("gcloud {}", args.join(" "));
        let result = self.run_api(args).await;
        match &result {
            Ok(response) => push_command_log(GcloudCommandLogEntry {
                command: cmd,
                response: response.clone(),
                success: true,
                exit_code: 0,
            }),
            Err(err) => push_command_log(GcloudCommandLogEntry {
                command: cmd,
                response: err.message.clone(),
                success: false,
                exit_code: err.exit_code,
            }),
        }
        result
    }
}

pub fn build_runner_from_preferences(preferences: &UserPreferences) -> Arc<dyn GcloudRunner> {
    if preferences.execution_mode == "api" {
        Arc::new(ApiRunner::new(
            preferences.project.clone(),
            preferences.api_access_token.clone(),
        )) as Arc<dyn GcloudRunner>
    } else {
        Arc::new(CliRunner::new(
            preferences.project.clone(),
            preferences.service_account_key_path.clone(),
        )) as Arc<dyn GcloudRunner>
    }
}

/// Locate the gcloud binary, checking common install paths on Windows.
fn find_gcloud_path() -> String {
    // First, try bare "gcloud" (works if it's on PATH)
    if which_exists("gcloud") {
        return "gcloud".to_string();
    }

    // On Windows, check known install locations
    #[cfg(target_os = "windows")]
    {
        use std::path::PathBuf;
        let candidates: Vec<PathBuf> = [
            std::env::var("LOCALAPPDATA").ok().map(|p| PathBuf::from(p).join("Google/Cloud SDK/google-cloud-sdk/bin/gcloud.cmd")),
            std::env::var("APPDATA").ok().map(|p| PathBuf::from(p).join("Google/Cloud SDK/google-cloud-sdk/bin/gcloud.cmd")),
            Some(PathBuf::from("C:/Program Files (x86)/Google/Cloud SDK/google-cloud-sdk/bin/gcloud.cmd")),
            Some(PathBuf::from("C:/Program Files/Google/Cloud SDK/google-cloud-sdk/bin/gcloud.cmd")),
            std::env::var("USERPROFILE").ok().map(|p| PathBuf::from(p).join("AppData/Local/Google/Cloud SDK/google-cloud-sdk/bin/gcloud.cmd")),
        ].into_iter().flatten().collect();

        for path in candidates {
            if path.exists() {
                return path.to_string_lossy().to_string();
            }
        }
    }

    // Fallback: return "gcloud" and let the error propagate naturally
    "gcloud".to_string()
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_ok()
}

#[async_trait]
impl GcloudRunner for CliRunner {
    async fn run(&self, args: &[&str]) -> Result<String, GcloudError> {
        let full_args = self.build_args(args);
        let cmd_str = format!("gcloud {}", full_args.join(" "));

        let mut command = Command::new(find_gcloud_path());
        command.args(&full_args);

        if let Some(ref cred_path) = self.credential_file {
            command.env("CLOUDSDK_AUTH_CREDENTIAL_FILE_OVERRIDE", cred_path);
        }

        let output = match command.output().await {
            Ok(output) => output,
            Err(e) => {
                let message = format!("Failed to execute gcloud: {e}");
                push_command_log(GcloudCommandLogEntry {
                    command: cmd_str.clone(),
                    response: message.clone(),
                    success: false,
                    exit_code: -1,
                });
                return Err(GcloudError {
                    message,
                    command: cmd_str,
                    exit_code: -1,
                });
            }
        };

        if output.status.success() {
            let response = String::from_utf8_lossy(&output.stdout).to_string();
            push_command_log(GcloudCommandLogEntry {
                command: cmd_str,
                response: response.clone(),
                success: true,
                exit_code: output.status.code().unwrap_or(0),
            });
            Ok(response)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            push_command_log(GcloudCommandLogEntry {
                command: cmd_str.clone(),
                response: stderr.clone(),
                success: false,
                exit_code: output.status.code().unwrap_or(-1),
            });
            Err(GcloudError {
                message: stderr,
                command: cmd_str,
                exit_code: output.status.code().unwrap_or(-1),
            })
        }
    }
}

/// Returns canned responses for testing, keyed by command prefix.
pub struct FakeRunner {
    responses: HashMap<String, Result<String, GcloudError>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    /// Register a canned successful response for commands starting with the given prefix.
    pub fn on_success(&mut self, prefix: &str, response: &str) {
        self.responses
            .insert(prefix.to_string(), Ok(response.to_string()));
    }

    /// Register a canned error response for commands starting with the given prefix.
    pub fn on_error(&mut self, prefix: &str, message: &str, exit_code: i32) {
        self.responses.insert(
            prefix.to_string(),
            Err(GcloudError {
                message: message.to_string(),
                command: prefix.to_string(),
                exit_code,
            }),
        );
    }
}

#[async_trait]
impl GcloudRunner for FakeRunner {
    async fn run(&self, args: &[&str]) -> Result<String, GcloudError> {
        let cmd = args.join(" ");
        for (prefix, result) in &self.responses {
            if cmd.starts_with(prefix) {
                match result {
                    Ok(response) => {
                        push_command_log(GcloudCommandLogEntry {
                            command: format!("gcloud {cmd}"),
                            response: response.clone(),
                            success: true,
                            exit_code: 0,
                        });
                    }
                    Err(err) => {
                        push_command_log(GcloudCommandLogEntry {
                            command: format!("gcloud {cmd}"),
                            response: err.message.clone(),
                            success: false,
                            exit_code: err.exit_code,
                        });
                    }
                }
                return result.clone();
            }
        }
        Err(GcloudError {
            message: format!("No canned response for: {cmd}"),
            command: cmd,
            exit_code: -1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_runner_builds_args_with_project() {
        let runner = CliRunner::new("my-proj".into(), None);
        let args = runner.build_args(&["compute", "disks", "list", "--zone=us-central1-a"]);
        assert!(args.contains(&"--project"));
        assert!(args.contains(&"my-proj"));
        assert!(args.contains(&"--format=json"));
    }

    #[test]
    fn cli_runner_skips_project_for_auth() {
        let runner = CliRunner::new("my-proj".into(), None);
        let args = runner.build_args(&["auth", "print-access-token"]);
        assert!(!args.contains(&"--project"));
        // auth commands still get --format=json
        assert!(args.contains(&"--format=json"));
    }

    #[test]
    fn cli_runner_skips_project_when_empty() {
        let runner = CliRunner::new(String::new(), None);
        let args = runner.build_args(&["compute", "disks", "list"]);
        assert!(!args.contains(&"--project"));
    }

    #[tokio::test]
    async fn fake_runner_returns_canned_success() {
        let mut runner = FakeRunner::new();
        runner.on_success("compute disks list", "[{\"name\": \"test\"}]");
        let result = runner.run(&["compute", "disks", "list"]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test"));
    }

    #[tokio::test]
    async fn fake_runner_returns_canned_error() {
        let mut runner = FakeRunner::new();
        runner.on_error("compute instances create", "quota exceeded", 1);
        let result = runner.run(&["compute", "instances", "create", "my-vm"]).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code, 1);
        assert!(err.message.contains("quota exceeded"));
    }

    #[tokio::test]
    async fn fake_runner_errors_on_unknown_command() {
        let runner = FakeRunner::new();
        let result = runner.run(&["unknown", "command"]).await;
        assert!(result.is_err());
    }

    #[test]
    fn logs_are_bounded_and_retrievable() {
        clear_command_logs();
        for i in 0..(MAX_LOG_ENTRIES + 5) {
            push_command_log(GcloudCommandLogEntry {
                command: format!("gcloud test {i}"),
                response: "ok".into(),
                success: true,
                exit_code: 0,
            });
        }

        let logs = get_command_logs();
        assert_eq!(logs.len(), MAX_LOG_ENTRIES);
        assert!(logs.last().is_some_and(|l| l.command.contains("test")));
    }
}
