use crate::gcloud::executor::build_runner_from_preferences;
use crate::models::config::DiskConfig;
use crate::models::machine::ConfigPreset;
use crate::models::UserPreferences;
use crate::state::AppState;
use tauri::State;

/// Resolve the config file path in the app data directory.
fn config_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cgelauncher");
    dir.join("preferences.json")
}

#[tauri::command]
pub async fn get_preferences(state: State<'_, AppState>) -> Result<UserPreferences, String> {
    let prefs = state.preferences.lock().await;
    Ok(prefs.clone())
}

#[tauri::command]
pub async fn set_preferences(
    state: State<'_, AppState>,
    preferences: UserPreferences,
) -> Result<UserPreferences, String> {
    // Rebuild the runner if project or credentials changed
    let mut prefs = state.preferences.lock().await;

    // Preserve OAuth tokens that the frontend doesn't track — the backend is the
    // authoritative store for tokens obtained via start_oauth_login / token refresh.
    let mut merged = preferences.clone();
    if merged.api_access_token.is_none() {
        merged.api_access_token = prefs.api_access_token.clone();
    }
    if merged.oauth_refresh_token.is_none() {
        merged.oauth_refresh_token = prefs.oauth_refresh_token.clone();
    }

    let runner_changed = prefs.project != merged.project
        || prefs.execution_mode != merged.execution_mode
        || prefs.api_access_token != merged.api_access_token;
    *prefs = merged.clone();
    drop(prefs);

    if runner_changed {
        let new_runner = build_runner_from_preferences(&merged);
        state.set_runner(new_runner).await;
    }

    persist_preferences(&merged)?;
    Ok(merged)
}

pub fn persist_preferences_pub(prefs: &UserPreferences) -> Result<(), String> {
    persist_preferences(prefs)
}

fn persist_preferences(prefs: &UserPreferences) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(prefs)
        .map_err(|e| format!("Failed to serialize preferences: {e}"))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write preferences to {}: {e}", path.display()))?;
    Ok(())
}

#[tauri::command]
pub async fn save_disk_config(
    state: State<'_, AppState>,
    disk_name: String,
    config: DiskConfig,
) -> Result<(), String> {
    let mut prefs = state.preferences.lock().await;
    prefs.disk_configs.insert(disk_name, config);
    let prefs_clone = prefs.clone();
    drop(prefs);
    persist_preferences(&prefs_clone)
}

#[tauri::command]
pub async fn get_disk_config(
    state: State<'_, AppState>,
    disk_name: String,
) -> Result<Option<DiskConfig>, String> {
    let prefs = state.preferences.lock().await;
    Ok(prefs.disk_configs.get(&disk_name).cloned())
}

#[tauri::command]
pub async fn save_custom_preset(
    state: State<'_, AppState>,
    preset: ConfigPreset,
) -> Result<Vec<ConfigPreset>, String> {
    let mut prefs = state.preferences.lock().await;
    // Replace if same name exists
    prefs.custom_presets.retain(|p| p.name != preset.name);
    prefs.custom_presets.push(preset);
    let prefs_clone = prefs.clone();
    drop(prefs);
    persist_preferences(&prefs_clone)?;
    Ok(prefs_clone.custom_presets)
}

#[tauri::command]
pub async fn delete_custom_preset(
    state: State<'_, AppState>,
    name: String,
) -> Result<Vec<ConfigPreset>, String> {
    use crate::models::machine::builtin_presets;

    let mut prefs = state.preferences.lock().await;
    let is_builtin = builtin_presets().iter().any(|p| p.name == name);
    if is_builtin {
        if !prefs.hidden_presets.contains(&name) {
            prefs.hidden_presets.push(name);
        }
    } else {
        prefs.custom_presets.retain(|p| p.name != name);
    }
    let prefs_clone = prefs.clone();
    drop(prefs);
    persist_preferences(&prefs_clone)?;
    Ok(prefs_clone.custom_presets)
}

/// Load preferences from disk, returning defaults if the file is missing or corrupt.
pub fn load_preferences() -> UserPreferences {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            eprintln!(
                "Warning: corrupt config file at {}, using defaults: {e}",
                path.display()
            );
            UserPreferences::default()
        }),
        Err(_) => UserPreferences::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_ends_with_preferences_json() {
        let p = config_path();
        assert!(p.ends_with("preferences.json"));
        assert!(p.to_string_lossy().contains("cgelauncher"));
    }

    #[test]
    fn load_returns_defaults_when_no_file() {
        // In a test environment, config file likely does not exist
        let prefs = load_preferences();
        // Should not panic and should return valid defaults
        assert!(prefs.default_spot);
    }
}
