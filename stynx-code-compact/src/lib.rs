pub mod auto_compact;
pub mod full_compact;
pub mod grouping;
pub mod micro_compact;
pub mod prompt;
pub mod session_memory_compact;
pub mod text;

pub use auto_compact::AutoCompactor;
pub use full_compact::FullCompactor;
pub use grouping::{MessageGroup, estimate_conversation_tokens};
pub use micro_compact::MicroCompactor;
pub use session_memory_compact::SessionMemoryCompactor;
pub use text::safe_truncate;

use stynx_code_errors::AppResult;
use stynx_code_types::{Conversation, Provider};

pub struct CompactionPipeline {
    auto: AutoCompactor,
    micro: MicroCompactor,
    session_memory: SessionMemoryCompactor,
    full: FullCompactor,
}

impl Default for CompactionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl CompactionPipeline {
    pub fn new() -> Self {
        Self {
            auto: AutoCompactor::default(),
            micro: MicroCompactor::default(),
            session_memory: SessionMemoryCompactor::new(),
            full: FullCompactor::new(),
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            auto: AutoCompactor::new(threshold),
            micro: MicroCompactor::default(),
            session_memory: SessionMemoryCompactor::new(),
            full: FullCompactor::new(),
        }
    }

    pub async fn compact(
        &self,
        conversation: Conversation,
        current_tokens: u64,
        token_limit: u64,
        provider: &dyn Provider,
    ) -> AppResult<(Conversation, Vec<String>)> {

        if !self.auto.should_compact(current_tokens, token_limit) {
            return Ok((conversation, vec![]));
        }

        let compacted = self.micro.compact_conversation(&conversation);

        let (memories, compacted) = self.session_memory.extract_and_compact(&compacted);

        let compacted = self.full.compact(&compacted, provider).await?;

        Ok((compacted, memories))
    }
}
