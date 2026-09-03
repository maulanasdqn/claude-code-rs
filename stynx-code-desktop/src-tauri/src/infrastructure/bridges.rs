use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use stynx_code_app::AppHandles;
use stynx_code_permission::{PromptBridge, PromptChoice, PromptRequest};
use stynx_code_tools::{QuestionBridge, QuestionRequest};
use tauri::AppHandle;

use crate::application::session::{PromptResponders, QuestionResponders, WorkspaceResponders};
use crate::domain::ui_event::UiEvent;
use crate::infrastructure::emitter::emit;

pub struct BridgeResponders {
    pub prompt: PromptResponders,
    pub question: QuestionResponders,
    pub workspace: WorkspaceResponders,
}

/// Installs the engine's interactive bridges (permission prompts, ask-user
/// questions, cross-workspace messages) and forwards each request to the
/// webview as a UiEvent; undeliverable requests resolve to a safe default.
pub fn install(app: &AppHandle, handles: &AppHandles) -> BridgeResponders {
    let responders = BridgeResponders {
        prompt: PromptResponders::default(),
        question: QuestionResponders::default(),
        workspace: WorkspaceResponders::default(),
    };
    let next_id = Arc::new(AtomicU64::new(1));

    let (prompt_bridge, mut prompt_rx) = PromptBridge::new();
    handles.permission.install_prompt_bridge(prompt_bridge);
    {
        let app = app.clone();
        let responders = responders.prompt.clone();
        let next_id = next_id.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(request) = prompt_rx.recv().await {
                let PromptRequest { tool_name, description, responder } = request;
                let id = next_id.fetch_add(1, Ordering::Relaxed);
                responders.lock().unwrap().insert(id, responder);
                let delivered =
                    emit(&app, UiEvent::PermissionRequest { id, tool_name, description });
                if !delivered
                    && let Some(responder) = responders.lock().unwrap().remove(&id)
                {
                    let _ = responder.send(PromptChoice::Deny);
                }
            }
        });
    }

    let (question_bridge, mut question_rx) = QuestionBridge::new();
    handles.ask_user_bridge.set(question_bridge);
    {
        let app = app.clone();
        let responders = responders.question.clone();
        let next_id = next_id.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(request) = question_rx.recv().await {
                let QuestionRequest { question, responder } = request;
                let id = next_id.fetch_add(1, Ordering::Relaxed);
                responders.lock().unwrap().insert(id, responder);
                let delivered = emit(&app, UiEvent::AskUserRequest { id, question });
                if !delivered
                    && let Some(responder) = responders.lock().unwrap().remove(&id)
                {
                    let _ = responder.send(None);
                }
            }
        });
    }

    let (workspace_bridge, mut workspace_rx) = stynx_code_app::WorkspaceBridge::new();
    handles.workspace_bridge.set(workspace_bridge);
    {
        let app = app.clone();
        let responders = responders.workspace.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(request) = workspace_rx.recv().await {
                let stynx_code_app::workspace_bridge::WorkspaceRequest { target, task, responder } =
                    request;
                let id = next_id.fetch_add(1, Ordering::Relaxed);
                responders.lock().unwrap().insert(id, responder);
                let delivered = emit(&app, UiEvent::WorkspaceMessageRequest { id, target, task });
                if !delivered
                    && let Some(responder) = responders.lock().unwrap().remove(&id)
                {
                    let _ = responder.send("(no UI connected to route the message)".to_string());
                }
            }
        });
    }

    responders
}
