use std::collections::HashSet;
use std::sync::Mutex;

pub trait TipProvider: Send + Sync {
    fn get_tip(&self) -> Option<String>;
    fn mark_shown(&self, tip_id: &str);
}

struct Tip {
    id: &'static str,
    text: &'static str,
}

const TIPS: &[Tip] = &[
    Tip { id: "shortcut_ctrl_c", text: "Press Ctrl+C to interrupt a running tool." },
    Tip { id: "slash_help", text: "Type /help to see all available slash commands." },
    Tip { id: "slash_compact", text: "Use /compact to summarize the conversation and free up context." },
    Tip { id: "multi_turn", text: "Claude remembers context across turns in the same session." },
    Tip { id: "tool_permission", text: "You can allow tools permanently with 'Always allow' to skip confirmation prompts." },
    Tip { id: "shift_tab", text: "Press Shift+Tab to toggle between single-line and multi-line input mode." },
    Tip { id: "slash_config", text: "Use /config to view or change settings during a session." },
    Tip { id: "memory", text: "Claude can remember things across sessions when you ask it to." },
];

pub struct StaticTipProvider {
    shown: Mutex<HashSet<String>>,
}

impl StaticTipProvider {
    pub fn new() -> Self {
        Self {
            shown: Mutex::new(HashSet::new()),
        }
    }
}

impl TipProvider for StaticTipProvider {
    fn get_tip(&self) -> Option<String> {
        let shown = self.shown.lock().unwrap_or_else(|e| e.into_inner());
        for tip in TIPS {
            if !shown.contains(tip.id) {
                return Some(tip.text.to_string());
            }
        }
        None
    }

    fn mark_shown(&self, tip_id: &str) {
        let mut shown = self.shown.lock().unwrap_or_else(|e| e.into_inner());
        shown.insert(tip_id.to_string());
    }
}
