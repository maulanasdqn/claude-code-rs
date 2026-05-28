use stynx_code_provider::AnthropicProvider;

use super::terminal::{BOLD, CYAN, DIM, GREEN, RED, RESET, YELLOW};

pub(super) async fn render_usage(anthropic: Option<&AnthropicProvider>) -> String {
    let Some(provider) = anthropic else {
        return format!("\n  {DIM}/usage is only available when the main agent is Claude.{RESET}\n");
    };
    if !provider.is_oauth() {
        return format!("\n  {DIM}/usage is only available for Claude AI subscribers (OAuth login){RESET}\n");
    }
    let utilization = match provider.fetch_usage().await {
        Ok(u) => u,
        Err(e) => return format!("\n  {BOLD}Error:{RESET} {e}\n"),
    };
    let mut out = format!("\n  {BOLD}{CYAN}Plan Usage{RESET}\n\n");
    if let Some(ref l) = utilization.five_hour { render_limit(&mut out, "Session limit (5 hours)", l); }
    if let Some(ref l) = utilization.seven_day { render_limit(&mut out, "Weekly limit (7 days)", l); }
    if let Some(ref l) = utilization.seven_day_opus { render_limit(&mut out, "Opus limit (7 days)", l); }
    if let Some(ref l) = utilization.seven_day_sonnet { render_limit(&mut out, "Sonnet limit (7 days)", l); }
    if let Some(ref extra) = utilization.extra_usage && extra.is_enabled {
        out.push_str(&format!("  {BOLD}Extra usage{RESET}\n"));
        if let (Some(used), Some(limit)) = (extra.used_credits, extra.monthly_limit) {
            out.push_str(&format!("  {DIM}Credits:{RESET} {CYAN}${used:.2}{RESET} / ${limit:.2}\n"));
        }
        if let Some(pct) = extra.utilization {
            out.push_str(&format!("  {DIM}Utilization:{RESET} {CYAN}{pct:.0}%{RESET}\n"));
        }
        out.push('\n');
    }
    if utilization.five_hour.is_none() && utilization.seven_day.is_none()
        && utilization.seven_day_opus.is_none() && utilization.seven_day_sonnet.is_none()
    {
        out.push_str(&format!("  {DIM}No usage data available.{RESET}\n"));
    }
    out
}

fn render_limit(out: &mut String, title: &str, limit: &stynx_code_provider::RateLimit) {
    let pct = limit.utilization.unwrap_or(0.0);
    let filled = (pct / 100.0 * 30.0).round() as usize;
    let empty = 30usize.saturating_sub(filled);
    let bar_color = if pct >= 90.0 { RED } else if pct >= 70.0 { YELLOW } else { GREEN };
    let bar = format!("{bar_color}{}{DIM}{}{RESET}", "█".repeat(filled), "░".repeat(empty));
    out.push_str(&format!("  {BOLD}{title}{RESET}\n"));
    out.push_str(&format!("  {bar}  {CYAN}{pct:.0}%{RESET} used\n"));
    if let Some(ref resets) = limit.resets_at {
        out.push_str(&format!("  {DIM}Resets {resets}{RESET}\n"));
    }
    out.push('\n');
}
