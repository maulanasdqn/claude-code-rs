pub trait TokenEstimator: Send + Sync {
    fn estimate_tokens(&self, text: &str) -> usize;
}

/// Simple estimator using ~4 characters per token heuristic.
pub struct SimpleEstimator;

impl SimpleEstimator {
    pub fn new() -> Self {
        Self
    }
}

impl TokenEstimator for SimpleEstimator {
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough approximation: ~4 characters per token for English text.
        // This intentionally rounds up to avoid underestimation.
        (text.len() + 3) / 4
    }
}
