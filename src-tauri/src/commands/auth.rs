use crate::gcloud::executor::ApiRunner;
use crate::gcloud::executor::build_runner_from_preferences;
use crate::models::config::AuthStatus;
use crate::oauth::{callback_server, flow};
use crate::state::AppState;
use reqwest::Client;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn check_auth(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let runner_guard = state.runner.lock().await;
    let runner = runner_guard.clone();
    drop(runner_guard);

    let mut status = crate::gcloud::auth::check_auth(&*runner)
        .await
        .map_err(|e| e.to_string())?;

    // Annotate the auth method based on stored preferences so the frontend
    // can distinguish gcloud CLI auth from OAuth API auth.
    if status.authenticated {
        let prefs = state.preferences.lock().await;
        if prefs.execution_mode == "api" {
            status.method = "oauth2".into();
        }
    }

    Ok(status)
}

/// Initiates the OAuth2 browser flow. Opens the system browser to Google's
/// consent screen, waits for the redirect callback, exchanges the code for
/// tokens, and stores them in preferences.
#[tauri::command]
pub async fn start_oauth_login(
    state: State<'_, AppState>,
    #[cfg_attr(target_os = "linux", allow(unused_variables))]
    app: tauri::AppHandle,
) -> Result<AuthStatus, String> {
    let pkce = flow::PkceChallenge::generate();
    let oauth_state = flow::random_state();
    let auth_url = flow::authorization_url(&oauth_state, &pkce.challenge);

    // Bind the callback port BEFORE opening the browser to avoid race conditions.
    let listener = callback_server::bind_callback_listener().await?;

    // Open the system browser. On Linux (WSL dev), always use powershell.exe
    // interop since shell.open targets the Linux env which has no browser.
    // On Windows and macOS, tauri shell.open works directly.
    #[cfg(target_os = "linux")]
    {
        let ps_ok = tokio::process::Command::new("powershell.exe")
            .args(["-Command", &format!("Start-Process '{auth_url}'")])
            .spawn()
            .is_ok();
        if !ps_ok {
            let _ = tokio::process::Command::new("xdg-open")
                .arg(&auth_url)
                .spawn();
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = tauri_plugin_shell::ShellExt::shell(&app).open(&auth_url, None);
    }

    // Wait for the redirect on localhost:7887 (2 minute timeout)
    let (code, returned_state) = callback_server::accept_callback(listener).await?;

    if returned_state != oauth_state {
        return Err("OAuth state mismatch — possible CSRF attack".into());
    }

    let client = Client::new();
    let tokens = flow::exchange_code(&client, &code, &pkce.verifier).await?;

    // Persist tokens and rebuild the runner
    let mut prefs = state.preferences.lock().await;
    prefs.api_access_token = Some(tokens.access_token.clone());
    if let Some(ref rt) = tokens.refresh_token {
        prefs.oauth_refresh_token = Some(rt.clone());
    }
    prefs.execution_mode = "api".into();
    let prefs_clone = prefs.clone();
    drop(prefs);

    let new_runner = Arc::new(match &prefs_clone.oauth_refresh_token {
        Some(rt) => ApiRunner::new_with_refresh(
            prefs_clone.project.clone(),
            tokens.access_token,
            rt.clone(),
        ),
        None => ApiRunner::new_with_token(prefs_clone.project.clone(), tokens.access_token),
    });
    state.set_runner(new_runner).await;

    crate::commands::config::persist_preferences_pub(&prefs_clone)?;

    Ok(AuthStatus {
        authenticated: true,
        method: "oauth2".into(),
        account: None,
    })
}

/// Revokes the stored OAuth refresh token and clears all OAuth credentials.
#[tauri::command]
pub async fn revoke_oauth(state: State<'_, AppState>) -> Result<(), String> {
    let mut prefs = state.preferences.lock().await;
    let refresh_token = prefs.oauth_refresh_token.clone();
    prefs.api_access_token = None;
    prefs.oauth_refresh_token = None;
    let prefs_clone = prefs.clone();
    drop(prefs);

    if let Some(token) = refresh_token {
        let client = Client::new();
        flow::revoke_token(&client, &token).await?;
    }

    let new_runner = build_runner_from_preferences(&prefs_clone);
    state.set_runner(new_runner).await;

    crate::commands::config::persist_preferences_pub(&prefs_clone)?;
    Ok(())
}
