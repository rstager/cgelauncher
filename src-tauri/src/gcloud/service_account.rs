use crate::gcloud::executor::GcloudError;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

const COMPUTE_SCOPE: &str = "https://www.googleapis.com/auth/compute";

#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    #[serde(rename = "type")]
    key_type: String,
    pub client_email: String,
    pub private_key: String,
    pub token_uri: String,
}

#[derive(Serialize)]
struct JwtClaims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: i64,
    exp: i64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
}

struct TokenCache {
    access_token: String,
    expires_at: i64,
}

/// Authenticates using a service account JSON key file, automatically
/// refreshing the OAuth access token before expiry.
pub struct ServiceAccountCredential {
    client_email: String,
    private_key: String,
    token_uri: String,
    cache: Mutex<Option<TokenCache>>,
}

impl ServiceAccountCredential {
    pub fn from_file(path: &str) -> Result<Self, GcloudError> {
        let contents = std::fs::read_to_string(path).map_err(|e| GcloudError {
            message: format!("Failed to read service account key file '{path}': {e}"),
            command: "load_service_account".into(),
            exit_code: 1,
        })?;

        let key: ServiceAccountKey =
            serde_json::from_str(&contents).map_err(|e| GcloudError {
                message: format!("Invalid service account key file '{path}': {e}"),
                command: "load_service_account".into(),
                exit_code: 1,
            })?;

        if key.key_type != "service_account" {
            return Err(GcloudError {
                message: format!(
                    "Expected key type 'service_account', got '{}'",
                    key.key_type
                ),
                command: "load_service_account".into(),
                exit_code: 1,
            });
        }

        Ok(Self {
            client_email: key.client_email,
            private_key: key.private_key,
            token_uri: key.token_uri,
            cache: Mutex::new(None),
        })
    }

    pub fn client_email(&self) -> &str {
        &self.client_email
    }

    pub async fn fetch_token(&self, client: &Client) -> Result<String, GcloudError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs() as i64;

        let mut cache = self.cache.lock().await;
        if let Some(ref cached) = *cache {
            if cached.expires_at - now > 60 {
                return Ok(cached.access_token.clone());
            }
        }

        let jwt = self.build_jwt(now)?;
        let response = client
            .post(&self.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| GcloudError {
                message: format!("Token exchange request failed: {e}"),
                command: "fetch_token".into(),
                exit_code: 401,
            })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(GcloudError {
                message: format!("Token exchange failed ({status}): {text}"),
                command: "fetch_token".into(),
                exit_code: 401,
            });
        }

        let token_response: TokenResponse =
            serde_json::from_str(&text).map_err(|e| GcloudError {
                message: format!("Failed to parse token response: {e}"),
                command: "fetch_token".into(),
                exit_code: 401,
            })?;

        *cache = Some(TokenCache {
            access_token: token_response.access_token.clone(),
            expires_at: now + token_response.expires_in,
        });

        Ok(token_response.access_token)
    }

    fn build_jwt(&self, now: i64) -> Result<String, GcloudError> {
        let header = Header::new(Algorithm::RS256);
        let claims = JwtClaims {
            iss: &self.client_email,
            scope: COMPUTE_SCOPE,
            aud: &self.token_uri,
            iat: now,
            exp: now + 3600,
        };
        let key = EncodingKey::from_rsa_pem(self.private_key.as_bytes()).map_err(|e| {
            GcloudError {
                message: format!("Invalid private key in service account file: {e}"),
                command: "build_jwt".into(),
                exit_code: 1,
            }
        })?;
        jsonwebtoken::encode(&header, &claims, &key).map_err(|e| GcloudError {
            message: format!("Failed to sign JWT: {e}"),
            command: "build_jwt".into(),
            exit_code: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_KEY: &str = r#"{
        "type": "service_account",
        "project_id": "test-project",
        "private_key_id": "key123",
        "private_key": "placeholder",
        "client_email": "test@test-project.iam.gserviceaccount.com",
        "token_uri": "https://oauth2.googleapis.com/token"
    }"#;

    #[test]
    fn parse_key_file_succeeds() {
        let key: ServiceAccountKey = serde_json::from_str(FIXTURE_KEY).unwrap();
        assert_eq!(key.client_email, "test@test-project.iam.gserviceaccount.com");
        assert_eq!(key.token_uri, "https://oauth2.googleapis.com/token");
        assert_eq!(key.key_type, "service_account");
    }

    #[test]
    fn parse_key_file_rejects_wrong_type() {
        let bad = FIXTURE_KEY.replace("\"service_account\"", "\"authorized_user\"");
        let key: ServiceAccountKey = serde_json::from_str(&bad).unwrap();

        // Simulate the from_file type check
        let result: Result<(), GcloudError> = if key.key_type != "service_account" {
            Err(GcloudError {
                message: format!("Expected key type 'service_account', got '{}'", key.key_type),
                command: "load_service_account".into(),
                exit_code: 1,
            })
        } else {
            Ok(())
        };
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("authorized_user"));
    }

    #[test]
    fn jwt_has_correct_claims() {
        let test_private_key = "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA2tAE9/FgZsdNig1uugU2/u8rDLd7t27USJhMmW6zITf6vqMZ
MldZ5O6Sy3muNWh3uQGKSZ6O3zZ3VRFWn0JzJlBUkIkrlo4mxI6aTng5j98tQytq
H7oGQ3r+YVCPVNdcsJcFUqTwOOLi1q+f/wzplYJJynr9BgB3mWh5Mfk7Pyw72h/u
tCM27BOfnBvEX4eIXth3YHZAxEgHT+og1Nt22A1IBovgDhw1FeQbvAqazRTjpQyA
7nGtkjSHhyiWQJmoMioBmNqxrlRe3FMyWcod1pr9aOPquCLN/1rb+v5LI6XJY82J
KZbFDRVOlNlfm12tQUlGiXLbGCvPetAZK8/EOwIDAQABAoIBAE2GQRRloT2Y5GiC
nNXsbhFBxJNjWMJjUnQL/auo1G9zCXRMjetPloptWnYb/PRwOGbEXG377uu3vOzX
UUTowqpy8Jsq9lYHtEWUrqgZoW9PZ5ZyRc3K11hGNeVdqQKqsOzh/OFMjc3h2POu
UrYHKaUufZ1vGMFtHfKs2K+zeWHGTsaiUsslewLDiRenhI5iGyFcWKmEi/wwogq/
0dAzw1AnOfbP5ybanOrvwGXVRCPDGbag/oklrWrx8qgQeugj8ykGEb+pO90QB4de
pbu3TCNxdtxS4DqfLZuRt4cDoVL2vXd5RmyaJxpG5sslFKqKixrLC4RXvgUl8emd
RzAAfXkCgYEA+y3GBDtB6wrxz7ndEzw3c93slUvOiSeUoi2PmrXqOEiXqhJhLHZN
YPF202lUdTtbf2Ign0jKySIVDjlucbRGJ2BJNWOlcT/eAhKZeEuE4JN4SvYwwIcd
kqQ3DRShHsvMxwN9yGmhXAYgN/TKo5VqSmOZhislP2Sn1/YBVejijmMCgYEA3wM0
8ClaJzC4/GKMtDyzvKN1Skam9kQLsu9XagMH+9TJGqpCv3hHRUm7/kyCDYuDq+rS
d2xOs29f7RceIuM02vTvjrTm+ldHCo1Q/0M1K/HKVQXP7dQVm3cyCmq8iaBgWZ81
LYJqP2s1qYv3KiWzd0UZ9Z2q8CwQypF9dZ/RTkkCgYBEEznRt8W70DGNXRBfwDg/
POx74hnN7l5IPhTnl1oteu8v9t9DT6TVG1xbG/b59uZrdcrloLLlJEmUm1gllPhW
f7AXujQCp46h/Sx+/+i5fP5jQqof4/7N2ZfaAbdRQ2byoS2b/ZTv/fEJeVzaTQqL
ssbPKC5mKf+bdl8SS5XrhQKBgQDbowvjL8bjbB/0KZcL9/DI62+bzIOpbRDclM5R
0VRumH3Lrj341xvSSLFG3dEESBBRI/9OsLO+EwW1upvqnjyzHKJGuTH3AjgsU0uf
a3CVrBeqrwO+5q61I6p8Ce1P2kyqV5uHC7daaFs8dWXi86iR4dOUTElLKwsKhkm6
q8D3SQKBgQDxuM5RFJGTg5oKQS1iOfd351iFmwkV/m3sAoPekJSf09VAddCbNvAb
gPFzn1hlIsOMWmr3gcRmEuDR3CfmHPeAebYgLFecKxzKoAwDAlL2X/KqxiQh90g5
wOTcr6jHxg1YzPcaAJI6vhrBYT3Wnw9Vm9of2q60jLCrOD2jxnBTKw==
-----END RSA PRIVATE KEY-----";

        let now = 1_700_000_000i64;
        let cred = ServiceAccountCredential {
            client_email: "test@example.iam.gserviceaccount.com".into(),
            private_key: test_private_key.into(),
            token_uri: "https://oauth2.googleapis.com/token".into(),
            cache: Mutex::new(None),
        };

        let jwt = cred.build_jwt(now).unwrap();
        // Decode without verification to inspect claims
        let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        validation.set_audience(&["https://oauth2.googleapis.com/token"]);
        validation.validate_exp = false;
        let token_data = jsonwebtoken::decode::<serde_json::Value>(
            &jwt,
            &jsonwebtoken::DecodingKey::from_secret(&[]),
            &validation,
        )
        .unwrap();

        let claims = token_data.claims;
        assert_eq!(claims["iss"], "test@example.iam.gserviceaccount.com");
        assert_eq!(claims["scope"], COMPUTE_SCOPE);
        assert_eq!(claims["exp"].as_i64().unwrap() - claims["iat"].as_i64().unwrap(), 3600);
    }
}
