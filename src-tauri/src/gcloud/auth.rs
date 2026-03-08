use crate::gcloud::executor::{GcloudError, GcloudRunner};
use crate::models::AuthStatus;

pub async fn check_auth(runner: &dyn GcloudRunner) -> Result<AuthStatus, GcloudError> {
    let result = runner.run(&["auth", "print-access-token"]).await;

    match result {
        Ok(_) => {
            // Token printed successfully; try to get the account name
            let account = match runner.run(&["auth", "list", "--filter=status:ACTIVE"]).await {
                Ok(json) => parse_active_account(&json),
                Err(_) => None,
            };

            Ok(AuthStatus {
                authenticated: true,
                method: "gcloud".into(),
                account,
            })
        }
        Err(_) => Ok(AuthStatus {
            authenticated: false,
            method: "gcloud".into(),
            account: None,
        }),
    }
}

fn parse_active_account(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v.as_array()?
        .first()?
        .get("account")?
        .as_str()
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcloud::executor::FakeRunner;

    #[tokio::test]
    async fn authenticated_user() {
        let mut runner = FakeRunner::new();
        runner.on_success("auth print-access-token", "ya29.token123");
        runner.on_success(
            "auth list",
            r#"[{"account": "user@example.com", "status": "ACTIVE"}]"#,
        );

        let status = check_auth(&runner).await.unwrap();
        assert!(status.authenticated);
        assert_eq!(status.method, "gcloud");
        assert_eq!(status.account.as_deref(), Some("user@example.com"));
    }

    #[tokio::test]
    async fn not_authenticated() {
        let mut runner = FakeRunner::new();
        runner.on_error("auth print-access-token", "no credentials", 1);

        let status = check_auth(&runner).await.unwrap();
        assert!(!status.authenticated);
        assert!(status.account.is_none());
    }

    #[test]
    fn parse_active_account_from_json() {
        let json = r#"[{"account": "test@proj.iam.gserviceaccount.com", "status": "ACTIVE"}]"#;
        assert_eq!(
            parse_active_account(json).as_deref(),
            Some("test@proj.iam.gserviceaccount.com")
        );
    }

    #[test]
    fn parse_active_account_empty_array() {
        assert_eq!(parse_active_account("[]"), None);
    }
}
