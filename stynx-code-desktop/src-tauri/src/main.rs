#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod domain;
mod infrastructure;

use application::session::SessionSlot;
use infrastructure::commands::{files, history, messaging, session, settings};

fn main() {
    tauri::Builder::default()
        .manage(SessionSlot::default())
        .invoke_handler(tauri::generate_handler![
            session::init_session,
            session::get_session_info,
            messaging::send_message,
            messaging::cancel,
            messaging::respond_permission,
            messaging::respond_ask_user,
            messaging::respond_workspace_message,
            settings::set_model,
            settings::set_mode,
            settings::set_thinking,
            settings::set_provider_key,
            history::list_sessions,
            history::load_session,
            history::new_session,
            history::delete_session,
            files::read_file,
            files::fetch_reference,
            files::list_project_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running stynx desktop");
}
