use claude_rust_errors::AppResult;
use claude_rust_types::{Conversation, Provider};

use crate::full_compact::FullCompactor;

/// Auto-compaction trigger based on context window utilization.
///
/// When the current token count exceeds a configurable threshold percentage
/// of the token limit, triggers a full compaction.
pub struct AutoCompactor {
    /// Threshold as a fraction (0.0 - 1.0). Default: 0.80 (80%).
    pub threshold: f64,
    full: FullCompactor,
}

impl Default for AutoCompactor {
    fn default() -> Self {
        Self {
            threshold: 0.80,
            full: FullCompactor::new(),
        }
    }
}

impl AutoCompactor {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            full: FullCompactor::new(),
        }
    }

    /// Returns `true` if the current token usage exceeds the threshold.
    pub fn should_compact(&self, current_tokens: u64, limit: u64) -> bool {
        if limit == 0 {
            return false;
        }
        (current_tokens as f64 / limit as f64) >= self.threshold
    }

    /// Compact the conversation via full compaction if the threshold is exceeded.
    pub async fn compact(
        &self,
        conversation: &Conversation,
        provider: &dyn Provider,
    ) -> AppResult<Conversation> {
        self.full.compact(conversation, provider).await
    }
}
