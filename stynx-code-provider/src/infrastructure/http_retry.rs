use reqwest::{RequestBuilder, Response};
use stynx_code_errors::{AppError, AppResult};

/// Send a request, retrying transient transport failures (connect/timeout/send
/// errors — e.g. flaky DNS or a dropped connection) with exponential backoff.
///
/// Only transport-level errors are retried; an HTTP error *status* (4xx/5xx) is
/// returned to the caller as a successful `Response` to handle. Status codes are
/// not retried here because that needs the response body (rate-limit headers).
pub(crate) async fn send_with_retry(builder: RequestBuilder, label: &str) -> AppResult<Response> {
    const MAX_ATTEMPTS: u32 = 4;

    for attempt in 1..=MAX_ATTEMPTS {
        // JSON bodies are always cloneable; if not, fall back to a single shot.
        let req = match builder.try_clone() {
            Some(r) => r,
            None => {
                return builder
                    .send()
                    .await
                    .map_err(|e| AppError::Provider(format!("{label} request failed: {e}")));
            }
        };

        match req.send().await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                let transient = e.is_connect() || e.is_timeout() || e.is_request();
                if attempt == MAX_ATTEMPTS || !transient {
                    return Err(AppError::Provider(format!(
                        "{label} request failed after {attempt} attempt(s): {e}"
                    )));
                }
                let backoff_ms = 300u64 * (1u64 << (attempt - 1)); // 300, 600, 1200ms
                tracing::warn!(
                    label = %label,
                    attempt,
                    error = %e,
                    "transient request failure; retrying in {backoff_ms}ms",
                );
                tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
            }
        }
    }

    // Loop always returns; this is unreachable.
    Err(AppError::Provider(format!("{label} request failed")))
}
