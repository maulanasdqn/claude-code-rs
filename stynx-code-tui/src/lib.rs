pub mod dialogs;
pub mod event;
pub mod layout;
pub mod render;
pub mod state;
pub mod theme;
pub mod tool_ui_impl;
pub mod widgets;

pub use event::event_handler::{EventHandler, UiAction};
pub use render::renderer::Renderer;
pub use state::{
    AppState, ConversationState, DialogOption, DisplayMessage, DisplayToolUse, InputMode,
    InputState, ModalKind, ModalState, PermissionChoice, SelectKind, ToolUseStatus,
};

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub state: AppState,
    in_alt: bool,
}

impl TuiApp {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal, state: AppState::new(), in_alt: true })
    }

    pub fn draw(&mut self) -> io::Result<()> {
        let Self { terminal, state, .. } = self;
        terminal.draw(|frame| Renderer::draw(frame, state))?;
        Ok(())
    }

    pub fn leave_alt(&mut self) {
        if self.in_alt {
            disable_raw_mode().ok();
            execute!(self.terminal.backend_mut(), DisableMouseCapture, DisableBracketedPaste, LeaveAlternateScreen).ok();
            self.in_alt = false;
        }
    }

    pub fn enter_alt(&mut self) {
        if !self.in_alt {
            enable_raw_mode().ok();
            execute!(self.terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste).ok();
            self.terminal.clear().ok();
            self.in_alt = true;
        }
    }

    pub fn is_in_alt(&self) -> bool { self.in_alt }

    pub fn tick_spinner(&mut self) {
        if self.state.is_streaming {
            self.state.spinner_tick = self.state.spinner_tick.wrapping_add(1);
            if self.state.spinner_tick % 8 == 0 {
                self.state.spinner_frame = self.state.spinner_frame.wrapping_add(1) % 10;
            }
        }
        self.state.toasts.tick();
    }

    pub fn sync_pause(&mut self, paused: &AtomicBool) {
        if paused.load(Ordering::Relaxed) {
            self.leave_alt();
        } else if !self.in_alt {
            self.enter_alt();
        }
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) { self.leave_alt(); }
}
