use std::time::Duration;

/// How many times a transient provider failure is retried before the turn
/// gives up. Combined with the capped backoff below this tolerates roughly
/// four minutes of sustained 529/429 weather.
pub const MAX_ATTEMPTS: u32 = 8;

const MAX_DELAY_MS: u64 = 60_000;
const JITTER_RANGE_MS: u64 = 1_000;

/// Transient provider failures worth retrying: Anthropic 529 overloaded,
/// 429 rate limits, and 5xx server-side hiccups. Auth, validation, and
/// permission errors must NOT match — retrying those just burns time.
pub fn is_retryable(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("overloaded")
        || lower.contains("529")
        || lower.contains("rate")
        || lower.contains("503")
        || lower.contains("500")
        || lower.contains("internal server")
        || lower.contains("service unavailable")
        || lower.contains("api_error")
}

/// Parses the `[retry_after_ms=N]` marker the providers prepend from the
/// server's `retry-after` header.
pub fn retry_after_ms(msg: &str) -> Option<u64> {
    let start = msg.find("[retry_after_ms=")?;
    let rest = &msg[start + "[retry_after_ms=".len()..];
    let end = rest.find(']')?;
    rest[..end].parse::<u64>().ok()
}

/// Delay before retry `attempt` (1-based): the server's retry-after when given,
/// otherwise exponential backoff (2s, 4s, 8s, …) — either way capped at 60s,
/// plus up to 1s of jitter so concurrent sub-agents don't stampede in lockstep.
pub fn retry_delay(attempt: u32, msg: &str) -> Duration {
    let base = retry_after_ms(msg).unwrap_or_else(|| 1_000u64.saturating_mul(2u64.saturating_pow(attempt)));
    let jitter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::from(d.subsec_nanos()) % JITTER_RANGE_MS)
        .unwrap_or(0);
    Duration::from_millis(base.min(MAX_DELAY_MS) + jitter)
}

/// First line of a provider error, shortened for display in a retry notice.
pub fn short_error(msg: &str) -> String {
    let line = msg.lines().next().unwrap_or("").trim();
    if line.chars().count() <= 120 {
        return line.to_string();
    }
    let mut out: String = line.chars().take(119).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_matches_transient_errors() {
        assert!(is_retryable("Overloaded"));
        assert!(is_retryable("HTTP 529 overloaded_error"));
        assert!(is_retryable("rate_limit_error: exceeded"));
        assert!(is_retryable("503 Service Unavailable"));
        assert!(!is_retryable("authentication_error: invalid x-api-key"));
        assert!(!is_retryable("invalid_request_error: max_tokens"));
    }

    #[test]
    fn honors_server_retry_after() {
        assert_eq!(retry_after_ms("[retry_after_ms=2500] 429 rate limited"), Some(2500));
        let d = retry_delay(1, "[retry_after_ms=2500] 429 rate limited");
        assert!(d >= Duration::from_millis(2500) && d < Duration::from_millis(2500 + JITTER_RANGE_MS));
    }

    #[test]
    fn backoff_is_capped() {
        let d = retry_delay(30, "overloaded");
        assert!(d <= Duration::from_millis(MAX_DELAY_MS + JITTER_RANGE_MS));
    }
}
