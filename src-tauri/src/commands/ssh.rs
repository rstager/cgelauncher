use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct SshConfigResponse {
    pub ssh_host: String,
    pub config_path: String,
}

#[tauri::command]
pub async fn configure_ssh(
    state: State<'_, AppState>,
    instance_name: String,
) -> Result<SshConfigResponse, String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    let project = prefs.project.clone();
    drop(prefs);

    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    let result =
        crate::gcloud::ssh::configure_ssh(&*runner, &instance_name, &zone, &project, None)
            .await
            .map_err(|e| e.to_string())?;

    Ok(SshConfigResponse {
        ssh_host: result.ssh_host,
        config_path: result.config_path,
    })
}

/// Spawn a terminal window running `ssh <host>` for the given instance.
///
/// The SSH host alias must already exist in ~/.ssh/config (written by configure_ssh
/// when the VM transitions to Running). Uses the system terminal emulator — no gcloud
/// binary required, so this works in both CLI and API execution modes.
#[tauri::command]
pub async fn launch_ssh_terminal(
    state: State<'_, AppState>,
    instance_name: String,
) -> Result<(), String> {
    let prefs = state.preferences.lock().await;
    let zone = prefs.zone.clone();
    let project = prefs.project.clone();
    drop(prefs);

    let ssh_host = format!("{instance_name}.{zone}.{project}");
    spawn_terminal_with_ssh(&ssh_host).map_err(|e| e.to_string())
}

/// Try terminal emulators in order until one spawns successfully.
///
/// Each attempt uses `.spawn()` (fire-and-forget). Returns Ok on the first
/// successful spawn, or an error if all candidates fail.
fn spawn_terminal_with_ssh(ssh_host: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "linux")]
    {
        // Under WSL, native Linux terminal emulators are unavailable.
        // Use wsl.exe -e to exec ssh directly (no shell interp, no PATH/alias issues).
        // Try Windows Terminal first, fall back to a plain cmd.exe console.
        if std::process::Command::new("wt.exe")
            .args(["new-tab", "wsl.exe", "-e", "ssh", ssh_host])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        if std::process::Command::new("cmd.exe")
            .args(["/c", "start", "cmd.exe", "/c", "wsl.exe", "-e", "ssh", ssh_host])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        // Fall back to native Linux terminal emulators (non-WSL Linux desktops)
        let candidates: &[(&str, &[&str])] = &[
            ("gnome-terminal", &["--", "ssh"]),
            ("xterm", &["-e", "ssh"]),
            ("konsole", &["-e", "ssh"]),
            ("xfce4-terminal", &["-e"]),
            ("lxterminal", &["-e"]),
        ];
        for (term, args) in candidates {
            let mut cmd = std::process::Command::new(term);
            if *term == "xfce4-terminal" || *term == "lxterminal" {
                cmd.args(*args).arg(format!("ssh {ssh_host}"));
            } else {
                cmd.args(*args).arg(ssh_host);
            }
            if cmd.spawn().is_ok() {
                return Ok(());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No supported terminal emulator found (tried wt.exe, cmd.exe, gnome-terminal, xterm, konsole, xfce4-terminal, lxterminal)",
        ))
    }

    #[cfg(target_os = "macos")]
    {
        // Open a new Terminal window running ssh via AppleScript
        let script = format!(
            r#"tell application "Terminal" to do script "ssh {}""#,
            ssh_host
        );
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        // Try Windows Terminal first, fall back to cmd.exe
        let wt = std::process::Command::new("wt.exe")
            .args(["new-tab", "ssh", ssh_host])
            .spawn();
        if wt.is_ok() {
            return Ok(());
        }
        std::process::Command::new("cmd.exe")
            .args(["/c", "start", "cmd.exe", "/k", "ssh", ssh_host])
            .spawn()?;
        Ok(())
    }
}
