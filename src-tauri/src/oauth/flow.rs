use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const CLIENT_ID: &str =
    env!("OAUTH_CLIENT_ID");
const CLIENT_SECRET: &str = env!("OAUTH_CLIENT_SECRET");
const REDIRECT_URI: &str = "http://localhost:7887/callback";
const SCOPE: &str = "https://www.googleapis.com/auth/compute";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const REVOKE_ENDPOINT: &str = "https://oauth2.googleapis.com/revoke";

#[derive(Debug, Clone)]
pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
}

impl PkceChallenge {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut bytes);
        let verifier = URL_SAFE_NO_PAD.encode(bytes);

        let digest = Sha256::digest(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(digest);

        Self { verifier, challenge }
    }
}

pub fn random_state() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn authorization_url(state: &str, code_challenge: &str) -> String {
    format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
         ?client_id={CLIENT_ID}\
         &redirect_uri={REDIRECT_URI}\
         &response_type=code\
         &scope={SCOPE}\
         &state={state}\
         &code_challenge={code_challenge}\
         &code_challenge_method=S256\
         &access_type=offline\
         &prompt=consent"
    )
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}

pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
}

pub async fn exchange_code(
    client: &Client,
    code: &str,
    code_verifier: &str,
) -> Result<OAuthTokens, String> {
    let response = client
        .post(TOKEN_ENDPOINT)
        .form(&[
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("code", code),
            ("code_verifier", code_verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {e}"))?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Token exchange failed ({status}): {text}"));
    }

    let token: TokenResponse =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse token response: {e}"))?;

    let now = now_secs();
    Ok(OAuthTokens {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires_at: now + token.expires_in,
    })
}

pub async fn refresh_access_token(
    client: &Client,
    refresh_token: &str,
) -> Result<OAuthTokens, String> {
    let response = client
        .post(TOKEN_ENDPOINT)
        .form(&[
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {e}"))?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Token refresh failed ({status}): {text}"));
    }

    let token: TokenResponse =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse refresh response: {e}"))?;

    let now = now_secs();
    Ok(OAuthTokens {
        access_token: token.access_token,
        // Refresh responses don't include a new refresh token
        refresh_token: None,
        expires_at: now + token.expires_in,
    })
}

pub async fn revoke_token(client: &Client, token: &str) -> Result<(), String> {
    let response = client
        .post(REVOKE_ENDPOINT)
        .form(&[("token", token)])
        .send()
        .await
        .map_err(|e| format!("Revoke request failed: {e}"))?;

    if !response.status().is_success() {
        // Treat revoke failures as non-fatal — token may already be invalid
        return Ok(());
    }
    Ok(())
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_and_challenge_are_different() {
        let pkce = PkceChallenge::generate();
        assert_ne!(pkce.verifier, pkce.challenge);
        assert!(!pkce.verifier.is_empty());
        assert!(!pkce.challenge.is_empty());
    }

    #[test]
    fn pkce_challenge_is_base64url() {
        let pkce = PkceChallenge::generate();
        // base64url chars: A-Z a-z 0-9 - _  (no + / =)
        assert!(pkce.challenge.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
        assert!(pkce.verifier.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn pkce_challenge_is_sha256_of_verifier() {
        let pkce = PkceChallenge::generate();
        let digest = Sha256::digest(pkce.verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(digest);
        assert_eq!(pkce.challenge, expected);
    }

    #[test]
    fn random_state_is_unique() {
        let s1 = random_state();
        let s2 = random_state();
        assert_ne!(s1, s2);
        assert!(!s1.is_empty());
    }

    #[test]
    fn authorization_url_contains_required_params() {
        let url = authorization_url("test-state", "test-challenge");
        assert!(url.contains("client_id="));
        assert!(url.contains("state=test-state"));
        assert!(url.contains("code_challenge=test-challenge"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("response_type=code"));
    }
}
