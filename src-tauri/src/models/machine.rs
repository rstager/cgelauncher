use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineConfig {
    pub machine_type: String,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u32>,
    pub spot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPreset {
    pub name: String,
    pub machine_type: String,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u32>,
    pub description: String,
}

pub fn builtin_presets() -> Vec<ConfigPreset> {
    vec![
        ConfigPreset {
            name: "Inference / Dev".into(),
            machine_type: "g2-standard-4".into(),
            gpu_type: None,
            gpu_count: None,
            description: "4 vCPU / 16 GB / 1x L4".into(),
        },
        ConfigPreset {
            name: "ML Training".into(),
            machine_type: "n1-standard-8".into(),
            gpu_type: Some("nvidia-tesla-t4".into()),
            gpu_count: Some(4),
            description: "8 vCPU / 30 GB / 4x T4".into(),
        },
        ConfigPreset {
            name: "A100 Training".into(),
            machine_type: "a2-highgpu-1g".into(),
            gpu_type: None,
            gpu_count: None,
            description: "12 vCPU / 85 GB / 1x A100 40GB".into(),
        },
        ConfigPreset {
            name: "CPU Only".into(),
            machine_type: "n1-standard-8".into(),
            gpu_type: None,
            gpu_count: None,
            description: "8 vCPU / 30 GB / No GPU".into(),
        },
    ]
}

/// Returns true for machine families where the GPU is inherent (A2, A3, G2).
pub fn has_builtin_gpu(machine_type: &str) -> bool {
    machine_type.starts_with("a2-")
        || machine_type.starts_with("a3-")
        || machine_type.starts_with("g2-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_presets_not_empty() {
        let presets = builtin_presets();
        assert!(presets.len() >= 4);
    }

    #[test]
    fn presets_have_required_fields() {
        for preset in builtin_presets() {
            assert!(!preset.name.is_empty());
            assert!(!preset.machine_type.is_empty());
            assert!(!preset.description.is_empty());
        }
    }

    #[test]
    fn a2_has_builtin_gpu() {
        assert!(has_builtin_gpu("a2-highgpu-1g"));
        assert!(has_builtin_gpu("a2-highgpu-2g"));
    }

    #[test]
    fn a3_has_builtin_gpu() {
        assert!(has_builtin_gpu("a3-highgpu-8g"));
    }

    #[test]
    fn n1_does_not_have_builtin_gpu() {
        assert!(!has_builtin_gpu("n1-standard-8"));
        assert!(!has_builtin_gpu("n1-highmem-16"));
    }

    #[test]
    fn machine_config_serializes() {
        let config = MachineConfig {
            machine_type: "n1-standard-8".into(),
            gpu_type: Some("nvidia-tesla-t4".into()),
            gpu_count: Some(4),
            spot: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("n1-standard-8"));
        assert!(json.contains("nvidia-tesla-t4"));
    }
}
