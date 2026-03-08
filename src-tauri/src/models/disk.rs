use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    pub name: String,
    pub size_gb: u64,
    pub status: String,
    pub disk_type: String,
    pub attached_to: Option<String>,
}

/// Raw JSON shape returned by `gcloud compute disks list --format=json`.
#[derive(Debug, Deserialize)]
struct GcloudDisk {
    name: String,
    #[serde(rename = "sizeGb", deserialize_with = "deserialize_string_u64")]
    size_gb: u64,
    status: String,
    /// Full URL like "projects/.../diskTypes/pd-ssd"
    #[serde(rename = "type")]
    disk_type_url: String,
    /// Array of instance self-link URLs; absent or empty when unattached.
    #[serde(default)]
    users: Vec<String>,
}

fn deserialize_string_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // gcloud sometimes returns sizeGb as a string, sometimes as a number
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        Str(String),
        Num(u64),
    }
    match StringOrNum::deserialize(deserializer)? {
        StringOrNum::Str(s) => s.parse().map_err(serde::de::Error::custom),
        StringOrNum::Num(n) => Ok(n),
    }
}

/// Extract the short disk type from a full URL.
/// e.g. "projects/my-proj/zones/us-central1-a/diskTypes/pd-ssd" -> "pd-ssd"
fn extract_disk_type(url: &str) -> String {
    url.rsplit('/').next().unwrap_or(url).to_string()
}

/// Extract the instance name from a full self-link URL.
/// e.g. "projects/my-proj/zones/us-central1-a/instances/my-vm" -> "my-vm"
fn extract_instance_name(url: &str) -> String {
    url.rsplit('/').next().unwrap_or(url).to_string()
}

pub fn parse_disks(json: &str) -> Result<Vec<Disk>, String> {
    let raw: Vec<GcloudDisk> =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse disk JSON: {e}"))?;

    Ok(raw
        .into_iter()
        .map(|d| Disk {
            name: d.name,
            size_gb: d.size_gb,
            status: d.status,
            disk_type: extract_disk_type(&d.disk_type_url),
            attached_to: d.users.first().map(|u| extract_instance_name(u)),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"[
        {
            "name": "ml-training-disk",
            "sizeGb": "200",
            "status": "READY",
            "type": "projects/my-proj/zones/us-central1-a/diskTypes/pd-ssd",
            "users": ["projects/my-proj/zones/us-central1-a/instances/ml-training-disk-vm"]
        },
        {
            "name": "dev-workspace",
            "sizeGb": "100",
            "status": "READY",
            "type": "projects/my-proj/zones/us-central1-a/diskTypes/pd-balanced"
        },
        {
            "name": "data-disk",
            "sizeGb": "500",
            "status": "CREATING",
            "type": "projects/my-proj/zones/us-central1-a/diskTypes/pd-standard",
            "users": []
        }
    ]"#;

    #[test]
    fn parses_attached_disk() {
        let disks = parse_disks(SAMPLE_JSON).unwrap();
        let d = &disks[0];
        assert_eq!(d.name, "ml-training-disk");
        assert_eq!(d.size_gb, 200);
        assert_eq!(d.status, "READY");
        assert_eq!(d.disk_type, "pd-ssd");
        assert_eq!(d.attached_to.as_deref(), Some("ml-training-disk-vm"));
    }

    #[test]
    fn parses_unattached_disk_missing_users() {
        let disks = parse_disks(SAMPLE_JSON).unwrap();
        let d = &disks[1];
        assert_eq!(d.name, "dev-workspace");
        assert_eq!(d.size_gb, 100);
        assert_eq!(d.disk_type, "pd-balanced");
        assert!(d.attached_to.is_none());
    }

    #[test]
    fn parses_unattached_disk_empty_users() {
        let disks = parse_disks(SAMPLE_JSON).unwrap();
        let d = &disks[2];
        assert_eq!(d.name, "data-disk");
        assert_eq!(d.size_gb, 500);
        assert_eq!(d.status, "CREATING");
        assert_eq!(d.disk_type, "pd-standard");
        assert!(d.attached_to.is_none());
    }

    #[test]
    fn parses_numeric_size_gb() {
        let json = r#"[{
            "name": "test",
            "sizeGb": 50,
            "status": "READY",
            "type": "projects/p/zones/z/diskTypes/pd-ssd"
        }]"#;
        let disks = parse_disks(json).unwrap();
        assert_eq!(disks[0].size_gb, 50);
    }

    #[test]
    fn rejects_invalid_json() {
        let result = parse_disks("not json");
        assert!(result.is_err());
    }

    #[test]
    fn empty_array_returns_empty_vec() {
        let disks = parse_disks("[]").unwrap();
        assert!(disks.is_empty());
    }

    #[test]
    fn extract_disk_type_from_url() {
        assert_eq!(
            extract_disk_type("projects/p/zones/z/diskTypes/pd-ssd"),
            "pd-ssd"
        );
    }

    #[test]
    fn extract_instance_name_from_url() {
        assert_eq!(
            extract_instance_name("projects/p/zones/z/instances/my-vm"),
            "my-vm"
        );
    }
}
