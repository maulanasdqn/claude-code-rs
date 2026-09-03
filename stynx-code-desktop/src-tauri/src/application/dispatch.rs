use std::sync::Arc;

use stynx_code_types::Message;

use crate::application::session::Session;
use crate::domain::ui_event::UiEvent;

/// Runs one engine turn for `message`, streaming events through `sink` and
/// persisting the result. Framework-free: the caller supplies the sink, so
/// this layer never touches Tauri directly.
pub fn dispatch<F>(session: Arc<Session>, message: Message, sink: F)
where
    F: Fn(UiEvent) + Send + Sync + Clone + 'static,
{
    let engine = session.engine.clone();
    let conversation = session.conversation.clone();
    let session_repo = session.session_repo.clone();
    let current_session_id = session.current_session_id.clone();
    let workspace_path = session.workspace_path.clone();
    let stream_sink = sink.clone();

    let handle = tokio::spawn(async move {
        let mut guard = conversation.lock().await;
        guard.push(message);
        let snapshot = guard.clone();

        let result = engine
            .run(snapshot, move |event| stream_sink(UiEvent::from(event)))
            .await;

        match result {
            Ok(updated) => {
                let compacted_mid_run = updated.messages.len() <= guard.messages.len();
                let existing = current_session_id.lock().unwrap().clone();
                if compacted_mid_run
                    && let Some(old_id) = &existing
                    && let Err(error) = session_repo.save(Some(old_id), &guard).await
                {
                    tracing::warn!("transcript archive before compact failed: {error}");
                }
                let target = match existing {
                    Some(id) if !compacted_mid_run => Some(id),
                    _ => session_repo.new_session_id().await.ok(),
                };
                match session_repo.save(target.as_deref(), &updated).await {
                    Ok(id) => *current_session_id.lock().unwrap() = Some(id),
                    Err(error) => tracing::warn!("session save failed: {error}"),
                }
                let captured_id = current_session_id.lock().unwrap().clone();
                stynx_code_truncus::capture(
                    captured_id.as_deref(),
                    &workspace_path,
                    &updated,
                    false,
                )
                .await;
                *guard = updated;
                sink(UiEvent::Idle);
            }
            Err(error) => {
                sink(UiEvent::Error { message: error.to_string() });
                sink(UiEvent::Idle);
            }
        }
    });

    *session.current_task.lock().unwrap() = Some(handle.abort_handle());
}
