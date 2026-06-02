//! Internal intern benchmark — `/intern-bench`.
//!
//! Runs a fixed suite of self-contained tasks against every configured intern,
//! grades each answer with the main model (LLM-as-judge) against a per-task
//! rubric, times each run, and produces a ranked leaderboard so you can see
//! which intern is actually the best for your setup.

use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::time::{Duration, Instant};

use stynx_code_config::HooksConfig;
use stynx_code_engine::{EngineEvent, QueryEngine};
use stynx_code_types::{
    ContentBlock, Conversation, Message, PermissionChecker, Provider, Role,
};
use stynx_code_tools::ToolRegistry;

use super::agent_tool::InternTool;

/// One graded task. `prompt` is what the intern sees; `rubric` is what the
/// judge uses to score the deliverable from 0–10.
pub struct BenchTask {
    pub id: &'static str,
    pub category: &'static str,
    pub prompt: &'static str,
    pub rubric: &'static str,
}

/// Tasks are deliberately self-contained (no repo state required) so the score
/// reflects the model's raw capability, not the contents of this checkout.
pub const BENCH_TASKS: &[BenchTask] = &[
    BenchTask {
        id: "palindrome",
        category: "coding",
        prompt: "Write a single Rust function with the exact signature \
`fn is_palindrome(s: &str) -> bool` that returns true if `s` reads the same \
forwards and backwards, ignoring ASCII case and skipping every character that \
is not alphanumeric. Output only the function — no main, no tests, no prose.",
        rubric: "10 = correct logic, ignores case, skips non-alphanumeric, \
compiles as-is, idiomatic Rust. Deduct for: wrong/edge-case bugs, not skipping \
punctuation, case sensitivity, won't compile, extraneous code beyond the function.",
    },
    BenchTask {
        id: "binary-search",
        category: "algorithm",
        prompt: "Implement binary search in Rust with the signature \
`fn binary_search(haystack: &[i32], needle: i32) -> Option<usize>` returning \
the index of `needle` if present, else None. The slice is sorted ascending. \
Output only the function.",
        rubric: "10 = O(log n), correct on empty/one-element/not-found/duplicate \
inputs, no overflow in midpoint calc, compiles. Deduct for off-by-one bugs, \
linear scan, overflow (lo+hi)/2 on large indices, won't compile.",
    },
    BenchTask {
        id: "find-bug",
        category: "debugging",
        prompt: "This Rust function should return the maximum of a slice but has \
a bug:\n\n```rust\nfn max(xs: &[i32]) -> i32 {\n    let mut m = 0;\n    for &x in xs {\n        if x > m { m = x; }\n    }\n    m\n}\n```\n\nState the bug in one sentence, then give the corrected function.",
        rubric: "10 = correctly identifies that initializing to 0 breaks all-negative \
slices (and ideally the empty-slice case), and the fix is correct (e.g. returns \
Option or initializes from the first element). Deduct for missing the real bug, \
wrong explanation, or a fix that still fails on negatives/empty.",
    },
    BenchTask {
        id: "git-undo",
        category: "instruction-following",
        prompt: "Output exactly the git command(s) to undo the most recent commit \
while KEEPING its changes staged in the index. Output only the command(s), one \
per line, with no explanation, no backticks, and no extra commands.",
        rubric: "10 = exactly `git reset --soft HEAD~1` (or equivalent like \
`git reset --soft HEAD^`), nothing else, no prose. Deduct heavily for `--hard`, \
`--mixed`/default reset (unstages), any extra commands, or added explanation \
that violates the 'commands only' instruction.",
    },
    BenchTask {
        id: "logic",
        category: "reasoning",
        prompt: "A bat and a ball cost $1.10 in total. The bat costs $1.00 more \
than the ball. How much does the ball cost? Give the number and a one-line \
justification.",
        rubric: "10 = $0.05 (5 cents) with correct reasoning (ball + bat where \
bat = ball + 1.00, so 2*ball + 1.00 = 1.10). Score 0 for the intuitive-wrong \
answer of $0.10. Deduct for right number but incoherent justification.",
    },
    BenchTask {
        id: "refactor",
        category: "refactor",
        prompt: "Simplify this Rust without changing behavior, and output only the \
simplified function:\n\n```rust\nfn count_even(xs: &[i32]) -> i32 {\n    let mut c = 0;\n    let mut i = 0;\n    while i < xs.len() {\n        if xs[i] % 2 == 0 {\n            c = c + 1;\n        }\n        i = i + 1;\n    }\n    return c;\n}\n```",
        rubric: "10 = idiomatic iterator chain (e.g. \
`xs.iter().filter(|x| *x % 2 == 0).count() as i32`), behavior preserved, \
compiles, no leftover imperative scaffolding. Deduct for behavior changes, \
non-compiling code, or only cosmetic edits.",
    },
];

const PER_TASK_TIMEOUT: Duration = Duration::from_secs(180);
const PASS_THRESHOLD: u8 = 6;

struct TaskResult {
    task_id: &'static str,
    category: &'static str,
    score: u8,
    secs: f64,
    note: String,
}

struct InternReport {
    name: String,
    results: Vec<TaskResult>,
}

impl InternReport {
    fn total_score(&self) -> u32 {
        self.results.iter().map(|r| r.score as u32).sum()
    }
    fn max_score(&self) -> u32 {
        (self.results.len() as u32) * 10
    }
    fn avg_score(&self) -> f64 {
        if self.results.is_empty() {
            0.0
        } else {
            self.total_score() as f64 / self.results.len() as f64
        }
    }
    fn passed(&self) -> usize {
        self.results.iter().filter(|r| r.score >= PASS_THRESHOLD).count()
    }
    fn total_secs(&self) -> f64 {
        self.results.iter().map(|r| r.secs).sum()
    }
}

/// Run the full benchmark and return a `(tui_summary, markdown_report)` pair.
///
/// `filter`, when set, restricts the run to interns whose label matches
/// (case-insensitive).
#[allow(clippy::too_many_arguments)]
pub async fn run_intern_bench(
    interns: &[Arc<InternTool>],
    judge_provider: Arc<dyn Provider>,
    judge_permission: Arc<dyn PermissionChecker>,
    judge_mode: Arc<AtomicU8>,
    judge_hooks: HooksConfig,
    filter: Option<&str>,
) -> (String, String) {
    let selected: Vec<&Arc<InternTool>> = interns
        .iter()
        .filter(|t| match filter {
            Some(f) => t.label().eq_ignore_ascii_case(f),
            None => true,
        })
        .collect();

    if selected.is_empty() {
        let msg = match filter {
            Some(f) => format!("no intern named '{f}'. run /intern-bench with no name to test all."),
            None => "no interns configured.".to_string(),
        };
        return (msg.clone(), msg);
    }

    let judge_engine = QueryEngine::new(
        judge_provider,
        Arc::new(ToolRegistry::new()),
        judge_permission,
        judge_mode,
        judge_hooks,
    )
    .with_max_turns(2);

    let mut reports: Vec<InternReport> = Vec::new();

    for intern in &selected {
        let mut results: Vec<TaskResult> = Vec::new();
        for task in BENCH_TASKS {
            let started = Instant::now();
            let output = match tokio::time::timeout(PER_TASK_TIMEOUT, intern.run_task(task.prompt)).await {
                Ok(Ok(out)) => out,
                Ok(Err(e)) => format!("[intern error] {e}"),
                Err(_) => "[timeout] intern did not finish in time".to_string(),
            };
            let secs = started.elapsed().as_secs_f64();
            let (score, note) = judge(&judge_engine, task, &output).await;
            results.push(TaskResult {
                task_id: task.id,
                category: task.category,
                score,
                secs,
                note,
            });
        }
        reports.push(InternReport {
            name: intern.label().to_string(),
            results,
        });
    }

    // Rank: highest average score first, ties broken by faster total time.
    reports.sort_by(|a, b| {
        b.avg_score()
            .partial_cmp(&a.avg_score())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.total_secs().partial_cmp(&b.total_secs()).unwrap_or(std::cmp::Ordering::Equal))
    });

    (format_summary(&reports), format_markdown(&reports))
}

/// Ask the judge model to score one deliverable. Returns `(score 0-10, note)`.
async fn judge(engine: &QueryEngine, task: &BenchTask, output: &str) -> (u8, String) {
    const JUDGE_SYSTEM: &str = "You are a strict, fair grader for an engineering \
benchmark. You score a single deliverable against a rubric on a 0–10 integer \
scale. Be calibrated: 10 is flawless, 6 is the minimum acceptable bar, 0 is \
wrong or empty. Ignore formatting noise — grade the actual deliverable. \
Respond with ONLY a JSON object: {\"score\": <int 0-10>, \"reason\": \"<one short sentence>\"}.";

    let user = format!(
        "TASK:\n{prompt}\n\nRUBRIC:\n{rubric}\n\nINTERN'S ANSWER:\n{output}\n\n\
Score the answer per the rubric. Respond with only the JSON object.",
        prompt = task.prompt,
        rubric = task.rubric,
        output = output,
    );

    let conv = Conversation {
        system: Some(JUDGE_SYSTEM.to_string()),
        messages: vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: user }],
        }],
    };

    let text = Arc::new(std::sync::Mutex::new(String::new()));
    let sink = text.clone();
    let run = engine.run(conv, move |event| {
        if let EngineEvent::TextDelta(t) = event {
            sink.lock().unwrap().push_str(&t);
        }
    });

    match tokio::time::timeout(PER_TASK_TIMEOUT, run).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return (0, format!("judge error: {e}")),
        Err(_) => return (0, "judge timed out".to_string()),
    }

    let raw = text.lock().unwrap().clone();
    parse_verdict(&raw)
}

/// Lenient parse of the judge's reply: prefer a JSON `{score, reason}`, else
/// fall back to the first integer 0–10 in the text.
fn parse_verdict(raw: &str) -> (u8, String) {
    if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        if end > start {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw[start..=end]) {
                let score = v.get("score").and_then(|s| s.as_i64()).unwrap_or(-1);
                if (0..=10).contains(&score) {
                    let reason = v
                        .get("reason")
                        .and_then(|r| r.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    return (score as u8, reason);
                }
            }
        }
    }

    // Fallback: first standalone integer in 0..=10.
    let mut num = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else if !num.is_empty() {
            if let Ok(n) = num.parse::<i64>() {
                if (0..=10).contains(&n) {
                    return (n as u8, raw.trim().chars().take(120).collect());
                }
            }
            num.clear();
        }
    }
    (0, format!("unparseable verdict: {}", raw.trim().chars().take(120).collect::<String>()))
}

fn medal(rank: usize) -> &'static str {
    match rank {
        0 => "🥇",
        1 => "🥈",
        2 => "🥉",
        _ => "  ",
    }
}

fn format_summary(reports: &[InternReport]) -> String {
    let mut out = String::new();
    let name_w = reports.iter().map(|r| r.name.len()).max().unwrap_or(8).max(8);

    out.push_str(&format!(
        "🏁 Intern Benchmark — {} intern(s) × {} tasks\n\n",
        reports.len(),
        BENCH_TASKS.len()
    ));

    out.push_str("Leaderboard (avg score, then speed):\n");
    for (i, r) in reports.iter().enumerate() {
        out.push_str(&format!(
            "  {medal} #{rank}  {name:<name_w$}  {avg:>4.1}/10   pass {pass}/{total}   {secs:>6.1}s\n",
            medal = medal(i),
            rank = i + 1,
            name = r.name,
            avg = r.avg_score(),
            pass = r.passed(),
            total = r.results.len(),
            secs = r.total_secs(),
        ));
    }

    // Per-task score matrix.
    out.push('\n');
    out.push_str(&format!("Per-task scores (/10):\n  {:<22}", "task"));
    for r in reports {
        out.push_str(&format!("{:>10}", r.name));
    }
    out.push('\n');
    for (ti, task) in BENCH_TASKS.iter().enumerate() {
        out.push_str(&format!("  {:<22}", format!("{} ({})", task.id, task.category)));
        for r in reports {
            let s = r.results.get(ti).map(|x| x.score).unwrap_or(0);
            out.push_str(&format!("{s:>10}"));
        }
        out.push('\n');
    }

    out.push_str("\nfull report → .stynx/intern-bench.md\n");
    out
}

fn format_markdown(reports: &[InternReport]) -> String {
    let mut md = String::new();
    md.push_str("# Intern Benchmark Report\n\n");
    md.push_str(&format!(
        "{} intern(s) graded across {} tasks by the main model (LLM-as-judge), 0–10 scale, pass ≥ {PASS_THRESHOLD}.\n\n",
        reports.len(),
        BENCH_TASKS.len()
    ));

    md.push_str("## Leaderboard\n\n");
    md.push_str("| Rank | Intern | Avg /10 | Total | Passed | Time |\n");
    md.push_str("|------|--------|---------|-------|--------|------|\n");
    for (i, r) in reports.iter().enumerate() {
        md.push_str(&format!(
            "| {} | {} | {:.1} | {}/{} | {}/{} | {:.1}s |\n",
            i + 1,
            r.name,
            r.avg_score(),
            r.total_score(),
            r.max_score(),
            r.passed(),
            r.results.len(),
            r.total_secs(),
        ));
    }

    md.push_str("\n## Per-task scores\n\n");
    md.push_str("| Task | Category |");
    for r in reports {
        md.push_str(&format!(" {} |", r.name));
    }
    md.push('\n');
    md.push_str("|------|----------|");
    for _ in reports {
        md.push_str("------|");
    }
    md.push('\n');
    for (ti, task) in BENCH_TASKS.iter().enumerate() {
        md.push_str(&format!("| {} | {} |", task.id, task.category));
        for r in reports {
            let s = r.results.get(ti).map(|x| x.score).unwrap_or(0);
            md.push_str(&format!(" {s} |"));
        }
        md.push('\n');
    }

    md.push_str("\n## Judge notes\n\n");
    for r in reports {
        md.push_str(&format!("### {}\n\n", r.name));
        for res in &r.results {
            md.push_str(&format!(
                "- **{}** ({}) — {}/10, {:.1}s — {}\n",
                res.task_id, res.category, res.score, res.secs, res.note
            ));
        }
        md.push('\n');
    }

    md
}
