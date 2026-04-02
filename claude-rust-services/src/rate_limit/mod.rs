#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub retry_after_ms: u64,
    pub limit_type: String,
    pub message: String,
}

pub fn format_rate_limit_message(info: &RateLimitInfo) -> String {
    let secs = info.retry_after_ms / 1000;
    if secs > 0 {
        format!(
            "Rate limited ({}): {}. Retry in {}s.",
            info.limit_type, info.message, secs
        )
    } else {
        format!("Rate limited ({}): {}", info.limit_type, info.message)
    }
}

pub fn parse_rate_limit_headers(status: u16, retry_after: Option<&str>) -> Option<RateLimitInfo> {
    if status != 429 {
        return None;
    }

    let retry_after_ms = retry_after
        .and_then(|v| v.parse::<f64>().ok())
        .map(|secs| (secs * 1000.0) as u64)
        .unwrap_or(60_000);

    Some(RateLimitInfo {
        retry_after_ms,
        limit_type: "api".to_string(),
        message: "Too many requests".to_string(),
    })
}
