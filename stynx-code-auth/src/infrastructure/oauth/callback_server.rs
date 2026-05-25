use stynx_code_errors::{AppError, AppResult};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Starts a one-shot HTTP server on `http://127.0.0.1:{port}/callback`,
/// waits for a single GET request with `?code=xxx`, returns the code.
pub async fn run_callback_server(port: u16) -> AppResult<String> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Provider(format!("failed to bind callback server on {addr}: {e}")))?;

    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| AppError::Provider(format!("callback server accept error: {e}")))?;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| AppError::Provider(format!("callback server read error: {e}")))?;

    let request = String::from_utf8_lossy(&buf[..n]);
    let code = extract_code(&request).ok_or_else(|| {
        AppError::Provider("no `code` parameter in OAuth callback request".to_string())
    })?;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|e| AppError::Provider(format!("callback server write error: {e}")))?;

    Ok(code)
}

fn extract_code(request: &str) -> Option<String> {
    let first_line = request.lines().next()?;
    // e.g. "GET /callback?code=abc123&state=xyz HTTP/1.1"
    let path = first_line.split_whitespace().nth(1)?;
    let query = path.split_once('?').map(|(_, q)| q)?;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("code=") {
            return Some(url_decode(value));
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hi = chars.next().unwrap_or('0');
            let lo = chars.next().unwrap_or('0');
            let hex = format!("{hi}{lo}");
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push(hi);
                result.push(lo);
            }
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    result
}
