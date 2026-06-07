use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;
use crate::widgets::footer::{fmt_elapsed_short, pretty_model, shrink_path};
use crate::widgets::spinner::FRAMES;

/// The bottom section of the left sidebar: model / session info, stacked
/// vertically (replaces the bottom powerline lualine).
pub struct SidebarInfo<'a> {
    pub cwd: &'a str,
    pub model: &'a str,
    pub mode: &'a str,
    pub cost: f64,
    pub git_branch: Option<&'a str>,
    pub is_streaming: bool,
    pub is_pending: bool,
    pub is_paused: bool,
    pub spinner_frame: usize,
    pub elapsed_secs: u64,
}

fn row(icon: &str, value: impl Into<String>, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {icon} "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.into(), Style::default().fg(theme::TEXT())),
    ])
}

impl<'a> Widget for SidebarInfo<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width < 4 {
            return;
        }
        let bg = theme::BACKGROUND();
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(Style::default().bg(bg));
            }
        }

        let w = area.width as usize;
        let mut lines: Vec<Line<'static>> = Vec::new();

        // Divider from the tools section above.
        lines.push(Line::from(Span::styled(
            "\u{2500}".repeat(w.saturating_sub(2)),
            Style::default().fg(theme::OVERLAY()),
        )));

        // Live status (only one of these at a time).
        if self.is_paused {
            lines.push(row("\u{23F8}", "paused", theme::GOLD()));
        } else if self.is_pending {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            lines.push(row(&ch.to_string(), "connecting…", theme::SUBTLE()));
        } else if self.is_streaming {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            lines.push(row(
                &ch.to_string(),
                format!("generating  {}", fmt_elapsed_short(self.elapsed_secs)),
                theme::FOAM(),
            ));
        }

        // Model.
        lines.push(row("\u{25C7}", pretty_model(self.model), theme::IRIS()));

        // Permission mode (hidden in Normal to stay clean).
        if self.mode != "Normal" {
            let (icon, color) = match self.mode {
                "Auto-accept" => ("\u{26A1}", theme::GOLD()),
                "Plan" => ("\u{25C6}", theme::IRIS()),
                "Bypass" => ("\u{26A0}", theme::LOVE()),
                _ => ("\u{25CF}", theme::FOAM()),
            };
            lines.push(row(icon, self.mode.to_string(), color));
        }

        // Git branch.
        if let Some(branch) = self.git_branch {
            lines.push(row("\u{E0A0}", branch.to_string(), theme::FOAM()));
        }

        // Working directory.
        lines.push(row(
            "\u{F07C}",
            shrink_path(self.cwd, w.saturating_sub(4)),
            theme::SUBTLE(),
        ));

        // Cost.
        lines.push(row("\u{0024}", format!("{:.4}", self.cost), theme::IRIS()));

        Paragraph::new(lines)
            .style(Style::default().bg(bg))
            .render(area, buf);
    }
}
