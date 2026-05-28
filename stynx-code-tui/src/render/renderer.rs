use ratatui::Frame;
use ratatui::style::Style;
use ratatui::widgets::Block;

use crate::layout::MainLayout;
use crate::state::{AppState, ModalKind};
use crate::theme;
use crate::widgets::delegate_bar::DelegateBar;
use crate::widgets::tool_detail::ToolDetail;
use crate::widgets::tool_history::{flat_rows, HistoryRow, ToolHistory};
use crate::widgets::{DialogSelect, Footer, InfoDialog, InputBox, InputDialog, MessageList, PermissionDialog, SlashPopover, SummaryBar, ThinkingPanel, ToastStack};

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self { Self }

    pub fn draw(frame: &mut Frame, state: &mut AppState) {
        let full = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(theme::BACKGROUND())),
            full,
        );

        let thinking_lines = if state.is_streaming && !state.live_thinking.trim().is_empty() {
            state
                .live_thinking
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count()
        } else {
            0
        };

        let delegate_lines = state.sub_agents.len();
        let has_summary = state.last_summary.is_some();
        let layout = MainLayout::split(
            full,
            state.input.line_count(),
            thinking_lines,
            delegate_lines,
            has_summary,
        );

        let tool_area = layout.tool_history;
        if let Some(area) = tool_area {
            frame.render_widget(ToolHistory::new(state), area);
        }

        frame.render_widget(
            MessageList::new(&mut state.conversation, state.spinner_frame)
                .with_tool_details(state.tool_details),
            layout.messages,
        );
        if let Some(thinking_area) = layout.thinking {
            frame.render_widget(
                ThinkingPanel::new(&state.live_thinking, state.spinner_frame),
                thinking_area,
            );
        }
        if let Some(delegate_area) = layout.delegate {
            frame.render_widget(
                DelegateBar::new(&state.sub_agents, state.spinner_frame),
                delegate_area,
            );
        }
        if let (Some(summary_area), Some(text)) = (layout.summary, state.last_summary.as_deref()) {
            frame.render_widget(SummaryBar::new(text), summary_area);
        }
        frame.render_widget(InputBox::new(&state.input, !state.is_streaming), layout.input);
        if !state.input.slash_matches.is_empty() {
            frame.render_widget(SlashPopover::new(&state.input, layout.input), full);
        }
        frame.render_widget(
            Footer {
                cwd: &state.cwd,
                model: &state.model_name,
                mode: &state.permission_mode,
                cost: state.total_cost,
                git_branch: state.git_branch.as_deref(),
                is_streaming: state.is_streaming,
                is_paused: state.is_paused,
                spinner_frame: state.spinner_frame,
            },
            layout.footer,
        );

        frame.render_widget(ToastStack::new(&state.toasts), full);

        if state.tool_history.detail_open {
            if let Some(idx) = state.tool_history.selected {
                let rows = flat_rows(state);
                if let Some(row) = rows.get(idx) {
                    let (mi, ti) = match row {
                        HistoryRow::Tool { msg, tool } => (*msg, *tool),
                        HistoryRow::Sub { msg, tool, .. } => (*msg, *tool),
                    };
                    let tool = &state.conversation.messages[mi].tool_uses[ti];
                    frame.render_widget(ToolDetail::new(tool), full);
                }
            }
        }

        match &state.modal.active {
            Some(ModalKind::Permission { tool_name, description, choice }) => {
                frame.render_widget(
                    PermissionDialog::new(tool_name, description, *choice),
                    full,
                );
            }
            Some(ModalKind::Select {
                title,
                query,
                options,
                selected,
                current_value,
                footer_hint,
                ..
            }) => {
                frame.render_widget(
                    DialogSelect::new(title, query, options, *selected)
                        .with_current(current_value.as_deref())
                        .with_footer(footer_hint.as_deref()),
                    full,
                );
            }
            Some(ModalKind::Info { title, rows }) => {
                frame.render_widget(InfoDialog::new(title, rows), full);
            }
            Some(ModalKind::Input { title, prompt, buffer, .. }) => {
                frame.render_widget(InputDialog::new(title, prompt, buffer), full);
            }
            Some(ModalKind::QuitConfirm) => {}
            None => {}
        }
    }
}

impl Default for Renderer {
    fn default() -> Self { Self }
}
