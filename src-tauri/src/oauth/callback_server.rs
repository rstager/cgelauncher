use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

const SUCCESS_HTML: &str = r#"<!DOCTYPE html><html><head><meta charset="utf-8"></head><body style="font-family:sans-serif;text-align:center;padding:60px">
<h2>&#x2713; Signed in successfully</h2><p>You can close this tab and return to GCE Launcher.</p>
</body></html>"#;

const ERROR_HTML: &str = r#"<!DOCTYPE html><html><body style="font-family:sans-serif;text-align:center;padding:60px">
<h2>Sign in failed</h2><p>An error occurred. Please close this tab and try again.</p>
</body></html>"#;

/// Binds the callback port immediately and returns a listener ready to accept.
/// Callers should bind first, then open the browser, then call [`accept_callback`].
pub async fn bind_callback_listener() -> Result<TcpListener, String> {
    // Bind to all interfaces so the Windows browser can reach this listener
    // when running under WSL (Windows localhost != WSL localhost).
    TcpListener::bind("0.0.0.0:7887")
        .await
        .map_err(|e| format!("Failed to bind callback port 7887: {e}"))
}

/// Waits for a single OAuth redirect on an already-bound listener.
/// Times out after 2 minutes.
pub async fn accept_callback(listener: TcpListener) -> Result<(String, String), String> {
    let (stream, _) = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        listener.accept(),
    )
    .await
    .map_err(|_| "OAuth login timed out after 2 minutes".to_string())?
    .map_err(|e| format!("Failed to accept connection: {e}"))?;

    let (reader, mut writer) = tokio::io::split(stream);
    let mut lines = BufReader::new(reader).lines();

    // Read the request line (e.g. "GET /callback?code=...&state=... HTTP/1.1")
    let request_line = lines
        .next_line()
        .await
        .map_err(|e| format!("Failed to read request: {e}"))?
        .ok_or("Empty request")?;

    let params = parse_callback_params(&request_line);

    let (status, body) = if params.contains_key("code") {
        ("200 OK", SUCCESS_HTML)
    } else {
        ("400 Bad Request", ERROR_HTML)
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = writer.write_all(response.as_bytes()).await;

    let code = params
        .get("code")
        .cloned()
        .ok_or("OAuth callback missing 'code' parameter")?;
    let state = params
        .get("state")
        .cloned()
        .ok_or("OAuth callback missing 'state' parameter")?;

    Ok((code, state))
}

fn parse_callback_params(request_line: &str) -> HashMap<String, String> {
    // "GET /callback?code=abc&state=xyz HTTP/1.1"
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("");

    let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");

    query
        .split('&')
        .filter_map(|part| {
            let (k, v) = part.split_once('=')?;
            Some((url_decode(k), url_decode(v)))
        })
        .collect()
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            if let Ok(byte) = u8::from_str_radix(&format!("{h1}{h2}"), 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_callback_extracts_code_and_state() {
        let line = "GET /callback?code=abc123&state=xyz456 HTTP/1.1";
        let params = parse_callback_params(line);
        assert_eq!(params.get("code").map(|s| s.as_str()), Some("abc123"));
        assert_eq!(params.get("state").map(|s| s.as_str()), Some("xyz456"));
    }

    #[test]
    fn parse_callback_handles_missing_query() {
        let line = "GET /callback HTTP/1.1";
        let params = parse_callback_params(line);
        assert!(params.is_empty());
    }

    #[test]
    fn url_decode_handles_percent_encoding() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a%2Bb"), "a+b");
    }

    #[test]
    fn url_decode_handles_plus_as_space() {
        assert_eq!(url_decode("hello+world"), "hello world");
    }
}
