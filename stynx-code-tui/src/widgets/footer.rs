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
    pub spinner_frame: usize,
}

fn shrink_path(cwd: &str, max: usize) -> String {
    if cwd.len() <= max {
        return cwd.to_string();
    }
    if let Some(home) = std::env::var_os("HOME") {
        if let Some(home) = home.to_str() {
            if cwd.starts_with(home) {
                let rest = &cwd[home.len()..];
                let candidate = format!("~{rest}");
                if candidate.len() <= max {
                    return candidate;
                }
                let take = max.saturating_sub(3);
                let start = candidate.len().saturating_sub(take);
                return format!("…{}", &candidate[start..]);
            }
        }
    }
    let take = max.saturating_sub(1);
    let start = cwd.len().saturating_sub(take);
    format!("…{}", &cwd[start..])
}

impl<'a> Widget for Footer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(theme::BACKGROUND()));
        }

        let max_path = (area.width as usize).saturating_sub(50).max(10);
        let path = shrink_path(self.cwd, max_path);

        let left = Line::from(Span::styled(
            path,
            Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND()),
        ));

        let mut right_spans: Vec<Span<'static>> = Vec::new();
        let sep = Span::styled("  ", Style::default().bg(theme::BACKGROUND()));

        if self.is_streaming {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            right_spans.push(Span::styled(
                format!("{ch} generating"),
                Style::default()
                    .fg(theme::PRIMARY())
                    .bg(theme::BACKGROUND())
                    .add_modifier(Modifier::ITALIC),
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
            format!("{mode_icon} {}", self.mode),
            Style::default().fg(mode_color).bg(theme::BACKGROUND()),
        ));
        right_spans.push(sep.clone());

        right_spans.push(Span::styled(
            self.model.to_string(),
            Style::default()
                .fg(theme::ACCENT())
                .bg(theme::BACKGROUND())
                .add_modifier(Modifier::BOLD),
        ));
        right_spans.push(sep.clone());

        right_spans.push(Span::styled(
            format!("${:.4}", self.cost),
            Style::default().fg(theme::PRIMARY()).bg(theme::BACKGROUND()),
        ));

        if let Some(branch) = self.git_branch {
            right_spans.push(sep);
            right_spans.push(Span::styled(
                format!("\u{E0A0} {branch}"),
                Style::default().fg(theme::SUCCESS()).bg(theme::BACKGROUND()),
            ));
        }

        let right = Line::from(right_spans);
        let right_width: u16 = right.width() as u16;
        let left_width: u16 = left.width() as u16;

        Paragraph::new(left)
            .style(Style::default().bg(theme::BACKGROUND()))
            .render(Rect { width: left_width.min(area.width), ..area }, buf);

        if area.width > right_width {
            let right_area = Rect {
                x: area.x + area.width - right_width,
                y: area.y,
                width: right_width,
                height: 1,
            };
            Paragraph::new(right)
                .style(Style::default().bg(theme::BACKGROUND()))
                .render(right_area, buf);
        }
    }
}
