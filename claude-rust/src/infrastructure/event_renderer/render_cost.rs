use super::super::terminal::{BOLD, CYAN, DIM, GREEN, RESET};

pub fn render_cost(input_tokens: u64, output_tokens: u64) {
    let input_cost = input_tokens as f64 * 3.0 / 1_000_000.0;
    let output_cost = output_tokens as f64 * 15.0 / 1_000_000.0;
    let total_cost = input_cost + output_cost;
    let total_tokens = input_tokens + output_tokens;

    println!();
    println!("  {DIM}┌─────────────────────────────────────┐{RESET}");
    println!("  {DIM}│{RESET} {BOLD}Token Usage{RESET}                        {DIM}│{RESET}");
    println!("  {DIM}├─────────────────────────────────────┤{RESET}");
    println!(
        "  {DIM}│{RESET}   Input    {CYAN}{input_tokens:>10}{RESET} {DIM}(${input_cost:.4}){RESET}  {DIM}│{RESET}"
    );
    println!(
        "  {DIM}│{RESET}   Output   {CYAN}{output_tokens:>10}{RESET} {DIM}(${output_cost:.4}){RESET}  {DIM}│{RESET}"
    );
    println!("  {DIM}├─────────────────────────────────────┤{RESET}");
    println!(
        "  {DIM}│{RESET}   Total    {BOLD}{total_tokens:>10}{RESET}  {GREEN}${total_cost:.4}{RESET}   {DIM}│{RESET}"
    );
    println!("  {DIM}└─────────────────────────────────────┘{RESET}");
    println!();
}
