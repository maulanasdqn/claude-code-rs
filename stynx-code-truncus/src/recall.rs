use crate::client::ApiClient;
use crate::config::Config;
use crate::dto::ContextBundle;
use crate::project::project_from_cwd;
use crate::util::fmt_date;

/// Build the session-start recall section for the given working directory, or
/// `None` when Truncus is unconfigured, unreachable, or has nothing to recall.
/// Best-effort: any error is logged at debug and swallowed so a memory outage
/// never blocks or slows down starting a Stynx session in a meaningful way.
pub async fn recall_section(cwd: &str) -> Option<String> {
    let cfg = Config::load()?;
    let project = project_from_cwd(cwd);
    let bundle = match ApiClient::new(&cfg).context(&project).await {
        Ok(bundle) => bundle,
        Err(e) => {
            tracing::debug!("truncus recall skipped: {e}");
            return None;
        }
    };
    if bundle.project_sessions.is_empty()
        && bundle.other_sessions.is_empty()
        && bundle.lessons.is_empty()
        && bundle.note_count == 0
    {
        return None;
    }
    Some(render(&project, &bundle))
}

fn render(project: &str, bundle: &ContextBundle) -> String {
    let mut text =
        String::from("Truncus memory — distilled summaries of your past sessions on this machine:\n");

    if !bundle.project_sessions.is_empty() {
        text.push_str(&format!("\n## Recent sessions in {project}\n"));
        for brief in &bundle.project_sessions {
            text.push_str(&format!(
                "\n### {} (session {})\n{}\n",
                fmt_date(brief.ended_at),
                brief.id,
                brief.summary
            ));
        }
    }

    if !bundle.other_sessions.is_empty() {
        text.push_str("\n## Recent sessions in other projects\n");
        for brief in &bundle.other_sessions {
            text.push_str(&format!(
                "\n### {} — {} (session {})\n{}\n",
                fmt_date(brief.ended_at),
                brief.project,
                brief.id,
                brief.summary
            ));
        }
    }

    if !bundle.lessons.is_empty() {
        text.push_str(&format!("\n## Lessons learned in {project} — apply these\n"));
        for lesson in &bundle.lessons {
            text.push_str(&format!(
                "\n- **[{}] {}** — {}\n",
                lesson.category, lesson.title, lesson.insight
            ));
        }
    }

    if bundle.note_count > 0 {
        text.push_str(&format!(
            "\n## Knowledge base\nThis project has a knowledge base of {} notes. Use the `knowledge_search` tool to pull relevant notes on demand instead of loading everything.\n",
            bundle.note_count
        ));
    }

    text.push_str(
        "\n## Grounding — say \"I don't know\" when unsure\nAnswer from this memory, the loaded lessons, the knowledge base (knowledge_search), and the actual code. If none support a specific claim — a file, API, name, number, or past decision — say so plainly instead of inventing specifics.\n",
    );
    text.push_str(
        "\nUse the memory tools (memory_search, recent_sessions, get_session, lessons, knowledge_search) for deeper recall.\n",
    );
    text
}
