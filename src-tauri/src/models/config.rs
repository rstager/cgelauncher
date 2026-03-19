use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::machine::{ConfigPreset, MachineConfig};

/// Per-disk saved config so the last-used machine/GPU config is restored on next launch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskConfig {
    pub machine_type: String,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u32>,
    pub spot: bool,
}

impl From<&MachineConfig> for DiskConfig {
    fn from(c: &MachineConfig) -> Self {
        Self {
            machine_type: c.machine_type.clone(),
            gpu_type: c.gpu_type.clone(),
            gpu_count: c.gpu_count,
            spot: c.spot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub project: String,
    pub zone: String,
    #[serde(alias = "execution_mode")]
    #[serde(default = "default_execution_mode")]
    pub execution_mode: String,
    #[serde(alias = "default_machine_type")]
    #[serde(default = "default_machine_type")]
    pub default_machine_type: Option<String>,
    #[serde(alias = "default_gpu_type")]
    #[serde(default)]
    pub default_gpu_type: Option<String>,
    #[serde(alias = "default_gpu_count")]
    #[serde(default)]
    pub default_gpu_count: Option<u32>,
    #[serde(alias = "default_spot")]
    #[serde(default = "default_spot")]
    pub default_spot: bool,
    #[serde(alias = "service_account_key_path")]
    #[serde(default)]
    pub service_account_key_path: Option<String>,
    #[serde(alias = "api_access_token")]
    #[serde(default)]
    pub api_access_token: Option<String>,
    #[serde(default)]
    pub oauth_refresh_token: Option<String>,
    #[serde(default)]
    pub custom_presets: Vec<ConfigPreset>,
    #[serde(default)]
    pub hidden_presets: Vec<String>,
    #[serde(default)]
    pub disk_configs: HashMap<String, DiskConfig>,
}

fn default_machine_type() -> Option<String> {
    Some("n1-standard-8".into())
}

fn default_spot() -> bool {
    true
}

fn default_execution_mode() -> String {
    "gcloud".into()
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            project: String::new(),
            zone: "us-central1-a".into(),
            execution_mode: default_execution_mode(),
            default_machine_type: default_machine_type(),
            default_gpu_type: None,
            default_gpu_count: None,
            default_spot: true,
            service_account_key_path: None,
            api_access_token: None,
            oauth_refresh_token: None,
            custom_presets: Vec::new(),
            hidden_presets: Vec::new(),
            disk_configs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub method: String,
    pub account: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.zone, "us-central1-a");
        assert_eq!(prefs.execution_mode, "gcloud");
        assert!(prefs.default_spot);
        assert!(prefs.project.is_empty());
    }

    #[test]
    fn deserializes_with_defaults() {
        let json = r#"{"project": "my-proj", "zone": "us-west1-b"}"#;
        let prefs: UserPreferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.project, "my-proj");
        assert_eq!(prefs.zone, "us-west1-b");
        assert_eq!(prefs.execution_mode, "gcloud");
        assert!(prefs.default_spot);
        assert_eq!(
            prefs.default_machine_type.as_deref(),
            Some("n1-standard-8")
        );
    }

    #[test]
    fn roundtrip_serialization() {
        let prefs = UserPreferences {
            project: "test-project".into(),
            zone: "europe-west4-a".into(),
            execution_mode: "api".into(),
            default_machine_type: Some("n1-standard-16".into()),
            default_gpu_type: Some("nvidia-tesla-v100".into()),
            default_gpu_count: Some(2),
            default_spot: false,
            service_account_key_path: Some("/path/to/key.json".into()),
            api_access_token: Some("token-123".into()),
            oauth_refresh_token: None,
            custom_presets: Vec::new(),
            hidden_presets: Vec::new(),
            disk_configs: HashMap::new(),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let parsed: UserPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.project, "test-project");
        assert_eq!(parsed.execution_mode, "api");
        assert!(!parsed.default_spot);
        assert_eq!(parsed.default_gpu_count, Some(2));
        assert_eq!(parsed.api_access_token.as_deref(), Some("token-123"));
    }

    #[test]
    fn auth_status_serializes() {
        let status = AuthStatus {
            authenticated: true,
            method: "gcloud".into(),
            account: Some("user@example.com".into()),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("user@example.com"));
    }
}
