use serde_json::Value;

pub fn format_permission_prompt(tool_name: &str, input: &Value) -> String {
    let input_str = input.to_string();
    let preview = if input_str.len() > 200 {
        format!("{}...", &input_str[..200])
    } else {
        input_str
    };
    format!("[permission] Tool \"{tool_name}\" wants to run: {preview} Allow? [y/n] ")
}
