use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;
use crate::widgets::spinner::FRAMES;

pub struct Footer<'a> {
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

fn shrink_path(cwd: &str, max: usize) -> String {
    let with_tilde = if let Some(home) = std::env::var_os("HOME") {
        if let Some(home) = home.to_str() {
            if cwd.starts_with(home) {
                format!("~{}", &cwd[home.len()..])
            } else {
                cwd.to_string()
            }
        } else {
            cwd.to_string()
        }
    } else {
        cwd.to_string()
    };

    if with_tilde.len() <= max {
        return with_tilde;
    }
    let take = max.saturating_sub(1);
    let start = with_tilde.len().saturating_sub(take);
    format!("…{}", &with_tilde[start..])
}

fn pretty_model(model: &str) -> String {
    let s = model.trim_start_matches("claude-");
    s.split('-').collect::<Vec<_>>().join("·")
}

fn fmt_elapsed_short(secs: u64) -> String {
    if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

impl<'a> Widget for Footer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = theme::SURFACE();
        let fg_dim = theme::SUBTLE();
        let dim_attr = Modifier::DIM;

        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(bg));
        }

        let max_path = (area.width as usize).saturating_sub(60).max(10);
        let path = shrink_path(self.cwd, max_path);

        let mut left_spans: Vec<Span<'static>> = vec![
            Span::styled(" ", Style::default().bg(bg)),
            Span::styled(
                path,
                Style::default().fg(theme::TEXT_MUTED()).bg(bg),
            ),
        ];

        if let Some(branch) = self.git_branch {
            left_spans.push(Span::styled(
                "  ",
                Style::default().fg(fg_dim).bg(bg).add_modifier(dim_attr),
            ));
            left_spans.push(Span::styled(
                "\u{E0A0} ",
                Style::default().fg(theme::SUCCESS()).bg(bg),
            ));
            left_spans.push(Span::styled(
                branch.to_string(),
                Style::default().fg(theme::SUCCESS()).bg(bg),
            ));
        }

        let mut right_spans: Vec<Span<'static>> = Vec::new();
        let sep = Span::styled(
            "  ",
            Style::default().fg(fg_dim).bg(bg).add_modifier(dim_attr),
        );

        if self.is_paused {
            right_spans.push(Span::styled(
                "⏸ ",
                Style::default().fg(theme::WARNING()).bg(bg).add_modifier(Modifier::BOLD),
            ));
            right_spans.push(Span::styled(
                "paused",
                Style::default().fg(theme::WARNING()).bg(bg).add_modifier(Modifier::ITALIC),
            ));
            right_spans.push(sep.clone());
        } else if self.is_pending {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            right_spans.push(Span::styled(
                format!("{ch} "),
                Style::default().fg(theme::SUBTLE()).bg(bg).add_modifier(Modifier::BOLD),
            ));
            right_spans.push(Span::styled(
                "connecting…",
                Style::default().fg(theme::SUBTLE()).bg(bg).add_modifier(Modifier::ITALIC),
            ));
            right_spans.push(sep.clone());
        } else if self.is_streaming {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            let elapsed_label = fmt_elapsed_short(self.elapsed_secs);
            right_spans.push(Span::styled(
                format!("{ch} "),
                Style::default().fg(theme::PRIMARY()).bg(bg).add_modifier(Modifier::BOLD),
            ));
            right_spans.push(Span::styled(
                format!("generating  {elapsed_label}"),
                Style::default().fg(theme::PRIMARY()).bg(bg).add_modifier(Modifier::ITALIC),
            ));
            right_spans.push(sep.clone());
        }

        let (mode_icon, mode_color) = match self.mode {
            "Auto-accept" => ("⚡", theme::WARNING()),
            "Plan" => ("◆", theme::PRIMARY()),
            "Bypass" => ("⚠", theme::ERROR()),
            _ => ("●", theme::ACCENT()),
        };
        right_spans.push(Span::styled(
            format!("{mode_icon} "),
            Style::default().fg(mode_color).bg(bg).add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::styled(
            self.mode.to_string(),
            Style::default().fg(mode_color).bg(bg),
        ));
        right_spans.push(sep.clone());

        right_spans.push(Span::styled(
            "◇ ",
            Style::default().fg(theme::IRIS()).bg(bg),
        ));
        right_spans.push(Span::styled(
            pretty_model(self.model),
            Style::default().fg(theme::IRIS()).bg(bg).add_modifier(Modifier::BOLD),
        ));
        right_spans.push(sep.clone());

        right_spans.push(Span::styled(
            format!("${:.4} ", self.cost),
            Style::default().fg(theme::GOLD()).bg(bg),
        ));

        let left = Line::from(left_spans);
        let right = Line::from(right_spans);
        let right_width: u16 = right.width() as u16;
        let left_width: u16 = left.width() as u16;

        Paragraph::new(left)
            .style(Style::default().bg(bg))
            .render(Rect { width: left_width.min(area.width), ..area }, buf);

        if area.width > right_width {
            let right_area = Rect {
                x: area.x + area.width - right_width,
                y: area.y,
                width: right_width,
                height: 1,
            };
            Paragraph::new(right)
                .style(Style::default().bg(bg))
                .render(right_area, buf);
        }
    }
}
