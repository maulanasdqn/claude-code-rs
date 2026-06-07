use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;
use crate::widgets::spinner::FRAMES;

/// Powerline separators (Nerd Font) — matches the user's tmux Rosé Pine bar.
const SEP_RIGHT: &str = "\u{E0B0}"; //

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

pub(super) fn shrink_path(cwd: &str, max: usize) -> String {
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

pub(super) fn pretty_model(model: &str) -> String {
    // Drop any provider prefix ("anthropic/…"), a leading "claude-" vendor word,
    // and a trailing date/build segment ("-20250514") so the bar stays compact:
    // e.g. "anthropic/claude-sonnet-4-20250514" -> "sonnet·4".
    let s = model.rsplit('/').next().unwrap_or(model);
    let s = s.trim_start_matches("claude-");
    s.split('-')
        .filter(|p| !(p.len() >= 6 && p.chars().all(|c| c.is_ascii_digit())))
        .collect::<Vec<_>>()
        .join("·")
}

pub(super) fn fmt_elapsed_short(secs: u64) -> String {
    if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

/// A single lualine/tmux-style segment: padded text on a solid color block.
struct Seg {
    text: String,
    fg: Color,
    bg: Color,
    bold: bool,
    italic: bool,
}

impl Seg {
    fn new(text: impl Into<String>, fg: Color, bg: Color) -> Self {
        Self { text: text.into(), fg, bg, bold: false, italic: false }
    }
    fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    fn style(&self) -> Style {
        let mut s = Style::default().fg(self.fg).bg(self.bg);
        if self.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        if self.italic {
            s = s.add_modifier(Modifier::ITALIC);
        }
        s
    }
}

/// Left-aligned bar: blocks grow rightward, joined with `` separators
/// that fade each block's bg into the next.
fn build_left(segs: &[Seg], bar_bg: Color) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, seg) in segs.iter().enumerate() {
        spans.push(Span::styled(format!(" {} ", seg.text), seg.style()));
        let next_bg = segs.get(i + 1).map(|s| s.bg).unwrap_or(bar_bg);
        spans.push(Span::styled(
            SEP_RIGHT,
            Style::default().fg(seg.bg).bg(next_bg),
        ));
    }
    Line::from(spans)
}

impl<'a> Widget for Footer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Match tmux `status-style 'bg=#191724 fg=#e0def4'` (base / text).
        let bar_bg = theme::BASE();
        let block_bg = theme::OVERLAY(); // #26233a — inactive / section blocks
        let accent = theme::IRIS(); // #c4a7e7 — active tab + section_z
        let base = theme::BASE();

        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(bar_bg));
        }

        // ---- left: [ mode? ]  [ path ]  [ branch ] ----
        let mut left_segs: Vec<Seg> = Vec::new();

        // Mode block — only for non-default modes (hidden in Normal so the bar
        // stays clean), colored like its shift+tab cycle.
        if self.mode != "Normal" {
            let (icon, color) = match self.mode {
                "Auto-accept" => ("\u{26A1}", theme::GOLD()), // ⚡
                "Plan" => ("\u{25C6}", theme::IRIS()),        // ◆
                "Bypass" => ("\u{26A0}", theme::LOVE()),      // ⚠
                _ => ("\u{25CF}", theme::FOAM()),             // ●
            };
            left_segs.push(Seg::new(format!("{icon} {}", self.mode), base, color).bold());
        }

        // Path is the "active window" block (iris bg, base fg, bold).
        let max_path = (area.width as usize).saturating_sub(46).max(10);
        left_segs.push(
            Seg::new(
                format!("\u{F07C} {}", shrink_path(self.cwd, max_path)),
                base,
                accent,
            )
            .bold(),
        );

        if let Some(branch) = self.git_branch {
            left_segs.push(Seg::new(
                format!("\u{E0A0} {branch}"),
                theme::FOAM(),
                block_bg,
            ));
        }

        // ---- right: [ status ]  [ model ]  [ cost ] ----
        let mut right_segs: Vec<Seg> = Vec::new();

        if self.is_paused {
            right_segs.push(
                Seg::new("\u{23F8} paused", theme::GOLD(), block_bg)
                    .bold()
                    .italic(),
            );
        } else if self.is_pending {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            right_segs.push(
                Seg::new(format!("{ch} connecting…"), theme::SUBTLE(), block_bg).italic(),
            );
        } else if self.is_streaming {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            let elapsed = fmt_elapsed_short(self.elapsed_secs);
            right_segs.push(
                Seg::new(
                    format!("{ch} generating  {elapsed}"),
                    theme::FOAM(),
                    block_bg,
                )
                .italic(),
            );
        }

        right_segs.push(
            Seg::new(
                format!("\u{25C7} {}", pretty_model(self.model)),
                theme::TEXT(),
                block_bg,
            )
            .bold(),
        );

        // Cost is the section_z block — iris, mirroring the path block.
        right_segs.push(Seg::new(format!("${:.4}", self.cost), base, accent).bold());

        // One contiguous left-aligned powerline strip at the bottom-left: mode,
        // path, branch, then status, model, cost — each in its own segment.
        left_segs.extend(right_segs);
        let bar = build_left(&left_segs, bar_bg);
        let bar_width = bar.width() as u16;

        Paragraph::new(bar)
            .style(Style::default().bg(bar_bg))
            .render(Rect { width: bar_width.min(area.width), ..area }, buf);
    }
}
