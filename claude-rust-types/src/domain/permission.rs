use std::sync::atomic::{AtomicU8, Ordering};

use claude_rust_errors::AppResult;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny(String),
}

/// Permission mode controls how dangerous tool permissions are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PermissionMode {
    /// Normal: prompt user for dangerous tools (default)
    Normal = 0,
    /// Auto-accept: automatically approve all dangerous tools
    AutoAccept = 1,
    /// Plan: only read-only tools available
    Plan = 2,
}

impl PermissionMode {
    /// Cycle to the next mode.
    pub fn next(self) -> Self {
        match self {
            Self::Normal => Self::AutoAccept,
            Self::AutoAccept => Self::Plan,
            Self::Plan => Self::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::AutoAccept => "Auto-accept",
            Self::Plan => "Plan",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Normal => "prompts for dangerous tools",
            Self::AutoAccept => "auto-approves all tools",
            Self::Plan => "read-only tools only",
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::AutoAccept,
            2 => Self::Plan,
            _ => Self::Normal,
        }
    }

    pub fn load(atomic: &AtomicU8) -> Self {
        Self::from_u8(atomic.load(Ordering::Relaxed))
    }

    pub fn store(self, atomic: &AtomicU8) {
        atomic.store(self as u8, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
pub trait PermissionChecker: Send + Sync {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision>;
}

pub struct AllowAll;

#[async_trait::async_trait]
impl PermissionChecker for AllowAll {
    async fn check(&self, _tool_name: &str, _input: &Value) -> AppResult<PermissionDecision> {
        Ok(PermissionDecision::Allow)
    }
}
