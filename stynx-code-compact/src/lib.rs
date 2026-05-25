pub mod auto_compact;
pub mod full_compact;
pub mod grouping;
pub mod micro_compact;
pub mod prompt;
pub mod session_memory_compact;

// Re-export key types
pub use auto_compact::AutoCompactor;
pub use full_compact::FullCompactor;
pub use grouping::MessageGroup;
pub use micro_compact::MicroCompactor;
pub use session_memory_compact::SessionMemoryCompactor;

use stynx_code_errors::AppResult;
use stynx_code_types::{Conversation, Provider};

/// The 4-stage compaction pipeline.
///
/// Stages:
/// 1. **Auto** — Checks if compaction is needed based on token threshold.
/// 2. **Micro** — Truncates oversized tool results.
/// 3. **Session Memory** — Extracts key memories before discarding content.
/// 4. **Full** — Sends conversation to a provider for summarization.
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

    /// Create a pipeline with a custom auto-compact threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            auto: AutoCompactor::new(threshold),
            micro: MicroCompactor::default(),
            session_memory: SessionMemoryCompactor::new(),
            full: FullCompactor::new(),
        }
    }

    /// Run the full pipeline: auto-check -> micro -> session_memory -> full.
    ///
    /// Returns `(compacted_conversation, extracted_memories)`.
    /// If compaction is not needed (below threshold), returns the original
    /// conversation unchanged with no memories.
    pub async fn compact(
        &self,
        conversation: Conversation,
        current_tokens: u64,
        token_limit: u64,
        provider: &dyn Provider,
    ) -> AppResult<(Conversation, Vec<String>)> {
        // 1. Check if compaction is needed
        if !self.auto.should_compact(current_tokens, token_limit) {
            return Ok((conversation, vec![]));
        }

        // 2. Micro-compact tool results
        let compacted = self.micro.compact_conversation(&conversation);

        // 3. Extract session memories
        let (memories, compacted) = self.session_memory.extract_and_compact(&compacted);

        // 4. Full compaction via provider
        let compacted = self.full.compact(&compacted, provider).await?;

        Ok((compacted, memories))
    }
}
