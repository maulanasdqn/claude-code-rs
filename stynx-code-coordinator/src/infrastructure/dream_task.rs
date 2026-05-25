use tokio::task::JoinHandle;

use stynx_code_errors::AppResult;

pub fn spawn_dream_task(description: &str) -> JoinHandle<AppResult<String>> {
    let desc = description.to_string();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(format!("Dream task completed: {}", desc))
    })
}
