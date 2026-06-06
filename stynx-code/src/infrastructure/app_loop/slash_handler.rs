use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_types::{Conversation, Provider};
use stynx_code_tui::TuiApp;

use crate::infrastructure::command_handler::handle_slash_command;
use crate::infrastructure::skills::Skill;
use crate::infrastructure::command_types::CommandAction;

pub(super) async fn handle_tui_slash(
    cmd: &str,
    provider: &Arc<dyn Provider>,
    anthropic: Option<&stynx_code_provider::AnthropicProvider>,
    config: &stynx_code_config::Settings,
    mode_flag: &Arc<AtomicU8>,
    system_prompt: &str,
    cwd: &str,
    conversation: &Conversation,
    skills: &[Skill],
    pinned_files: &mut Vec<String>,
    tui: &mut TuiApp,
) -> Option<CommandAction> {
    use crate::infrastructure::app_actions::{copy_last_response, handle_add, show_files, show_skills};
    use crate::infrastructure::app_help::print_help;

    match cmd {
        "/quit" | "/exit" => return Some(CommandAction::Quit),
        "/version" => {
            tui.state.push_system_message(format!("stynx-code v{}", env!("CARGO_PKG_VERSION")));
            return None;
        }
        "/files" => { show_files(pinned_files); return None; }
        "/copy" => { copy_last_response(conversation); return None; }
        "/help" => {
            tui.leave_alt();
            print_help(skills);
            let _ = std::io::stdin().read_line(&mut String::new());
            tui.enter_alt();
            return None;
        }
        "/skills" => {
            tui.leave_alt();
            show_skills(skills);
            let _ = std::io::stdin().read_line(&mut String::new());
            tui.enter_alt();
            return None;
        }
        _ => {}
    }
    if let Some(path) = cmd.strip_prefix("/add ") {
        handle_add(path, pinned_files);
        return None;
    }

    tui.leave_alt();
    let result = handle_slash_command(
        cmd, &**provider, anthropic, config, mode_flag,
        system_prompt, cwd, conversation, skills,
    ).await;
    tui.enter_alt();
    result
}
