use super::super::terminal::{BOLD, DIM, GREEN, RESET};

pub fn render_cost(input_tokens: u64, output_tokens: u64) {
    let total_cost = cost(input_tokens, output_tokens);
    let total_tokens = input_tokens + output_tokens;
    println!();
    println!("  {DIM}↗  {total_tokens} tokens  ({input_tokens} in · {output_tokens} out){RESET}   {GREEN}${total_cost}{RESET}");
    println!();
}

pub fn render_exit_summary(input_tokens: u64, output_tokens: u64) {
    let w = crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80).saturating_sub(4).max(10);
    let total_tokens = input_tokens + output_tokens;
    let total_cost = cost(input_tokens, output_tokens);
    println!();
    println!("  {DIM}{}{RESET}", "─".repeat(w));
    println!("  {DIM}Total:{RESET} {BOLD}{}{RESET} {DIM}tokens  ·  {RESET}{GREEN}${total_cost}{RESET}", fmt_tokens(total_tokens));
    println!("  {DIM}Goodbye!{RESET}");
    println!();
}

fn cost(input: u64, output: u64) -> String {
    let c = input as f64 * 3.0 / 1_000_000.0 + output as f64 * 15.0 / 1_000_000.0;
    if c > 0.50 { format!("{c:.2}") } else { format!("{c:.4}") }
}

fn fmt_tokens(n: u64) -> String {
    if n >= 1000 { format!("{:.1}K", n as f64 / 1000.0) } else { format!("{n}") }
}
