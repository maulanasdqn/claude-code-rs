use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use stynx_code_errors::{AppError, AppResult};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub fn generate_state() -> String {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .expect("OS CSPRNG unavailable — cannot generate OAuth state safely");
    URL_SAFE_NO_PAD.encode(bytes)
}

pub async fn run_callback_server(port: u16, expected_state: &str) -> AppResult<String> {
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
    let params = parse_query_params(&request);

    let returned_state = params.iter()
        .find_map(|(k, v)| if k == "state" { Some(v.as_str()) } else { None })
        .ok_or_else(|| AppError::Provider(
            "missing `state` parameter in OAuth callback (CSRF protection violated)".to_string()
        ))?;

    if !constant_time_eq(returned_state.as_bytes(), expected_state.as_bytes()) {
        return Err(AppError::Provider(
            "OAuth `state` mismatch — possible CSRF attack, refusing to continue".to_string(),
        ));
    }

    let code = params.iter()
        .find_map(|(k, v)| if k == "code" { Some(v.clone()) } else { None })
        .ok_or_else(|| AppError::Provider("no `code` parameter in OAuth callback".to_string()))?;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|e| AppError::Provider(format!("callback server write error: {e}")))?;

    Ok(code)
}

fn parse_query_params(request: &str) -> Vec<(String, String)> {
    let Some(first_line) = request.lines().next() else { return Vec::new(); };
    let Some(path) = first_line.split_whitespace().nth(1) else { return Vec::new(); };
    let Some(query) = path.split_once('?').map(|(_, q)| q) else { return Vec::new(); };
    query
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            Some((k.to_string(), url_decode(v)))
        })
        .collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut acc = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
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
