use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::layout::MainLayout;
use crate::state::{AppState, ModalKind};
use crate::theme;
use crate::widgets::{InputBox, MessageList, PermissionDialog, Spinner, StatusBar};

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self { Self }

    pub fn draw(frame: &mut Frame, state: &mut AppState) {
        let full = frame.area();
        frame.render_widget(Block::default().style(Style::default().bg(theme::BASE)), full);

        let (msg, stream_row, inp, stat) = if state.is_streaming {
            let c = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ]).split(full);
            (c[0], Some(c[1]), c[2], c[3])
        } else {
            let [m, i, s] = MainLayout::split(full);
            (m, None, i, s)
        };

        frame.render_widget(MessageList::new(&mut state.conversation), msg);

        if let Some(area) = stream_row {
            let ch = Spinner::new(state.spinner_frame).current_char();
            let line = Line::from(vec![
                Span::styled(format!("  {} ", ch), Style::default().fg(theme::IRIS).add_modifier(Modifier::BOLD)),
                Span::styled("Generating response", Style::default().fg(theme::MUTED).add_modifier(Modifier::ITALIC)),
                Span::styled("...", Style::default().fg(theme::MUTED)),
            ]);
            frame.render_widget(Paragraph::new(line).style(Style::default().bg(theme::BASE)), area);
        }

        frame.render_widget(InputBox::new(&state.input, !state.is_streaming), inp);

        frame.render_widget(
            StatusBar::new(&state.model_name, &state.permission_mode, state.total_cost, state.git_branch.as_deref()),
            stat,
        );

        if let Some(ModalKind::Permission { tool_name, description }) = &state.modal.active {
            frame.render_widget(PermissionDialog::new(tool_name, description), full);
        }
    }
}

impl Default for Renderer {
    fn default() -> Self { Self }
}
