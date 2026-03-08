/// Embedded auto-stop startup script injected into VMs via GCE metadata.
/// The script installs a systemd timer that monitors SSH sessions and
/// shuts down the VM after a configurable idle timeout (default: 30 min).
pub const STARTUP_SCRIPT: &str = include_str!("auto_stop_startup.sh");
