use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::layout::MainLayout;
use crate::state::{AppState, ModalKind};
use crate::widgets::{InputBox, MessageList, PermissionDialog, Spinner, StatusBar};

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self { Self }

    pub fn draw(frame: &mut Frame, state: &AppState) {
        let full = frame.area();
        let [msg, inp, stat] = MainLayout::split(full);

        frame.render_widget(MessageList::new(&state.conversation), msg);

        if state.is_streaming {
            let chunks = Layout::horizontal([
                Constraint::Length(4),
                Constraint::Min(1),
            ]).split(inp);
            frame.render_widget(Spinner::new(state.spinner_frame), chunks[0]);
            frame.render_widget(InputBox::new(&state.input, false), chunks[1]);
        } else {
            frame.render_widget(InputBox::new(&state.input, true), inp);
        }

        frame.render_widget(
            StatusBar::new(
                &state.model_name,
                &state.permission_mode,
                state.total_cost,
                state.git_branch.as_deref(),
            ),
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
