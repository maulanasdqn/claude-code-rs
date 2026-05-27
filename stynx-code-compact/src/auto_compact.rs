use stynx_code_errors::AppResult;
use stynx_code_types::{Conversation, Provider};

use crate::full_compact::FullCompactor;

pub struct AutoCompactor {

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

    pub fn should_compact(&self, current_tokens: u64, limit: u64) -> bool {
        if limit == 0 {
            return false;
        }
        (current_tokens as f64 / limit as f64) >= self.threshold
    }

    pub async fn compact(
        &self,
        conversation: &Conversation,
        provider: &dyn Provider,
    ) -> AppResult<Conversation> {
        self.full.compact(conversation, provider).await
    }
}
