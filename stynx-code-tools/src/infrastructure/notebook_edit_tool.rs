use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct NotebookEditTool;

impl NotebookEditTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for NotebookEditTool {
    fn name(&self) -> &str {
        "notebook_edit"
    }

    fn description(&self) -> &str {
        "Edit a Jupyter notebook (.ipynb) file by inserting, replacing, or deleting cells."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "notebook_path": {
                    "type": "string",
                    "description": "Path to the .ipynb notebook file"
                },
                "operation": {
                    "type": "string",
                    "description": "Operation to perform: insert_cell, replace_cell, or delete_cell",
                    "enum": ["insert_cell", "replace_cell", "delete_cell"]
                },
                "cell_index": {
                    "type": "integer",
                    "description": "Index of the cell to operate on (0-based)"
                },
                "cell_type": {
                    "type": "string",
                    "description": "Type of cell (for insert_cell): code or markdown",
                    "enum": ["code", "markdown"]
                },
                "source": {
                    "type": "string",
                    "description": "Source content for the cell (for insert_cell and replace_cell)"
                }
            },
            "required": ["notebook_path", "operation", "cell_index"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn is_destructive(&self, _input: &Value) -> bool { true }

    fn get_path(&self, input: &Value) -> Option<String> {
        input.get("notebook_path").and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let notebook_path = input
            .get("notebook_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'notebook_path' field".into()))?;

        let operation = input
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'operation' field".into()))?;

        let cell_index = input
            .get("cell_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AppError::Tool("missing 'cell_index' field".into()))? as usize;

        tracing::info!(notebook_path, operation, cell_index, "editing notebook");

        let content = tokio::fs::read_to_string(notebook_path)
            .await
            .map_err(|e| AppError::Tool(format!("cannot read '{notebook_path}': {e}")))?;

        let mut notebook: Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Tool(format!("invalid notebook JSON: {e}")))?;

        let cells = notebook
            .get_mut("cells")
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| AppError::Tool("notebook has no 'cells' array".into()))?;

        let result = match operation {
            "insert_cell" => {
                let cell_type = input
                    .get("cell_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("code");

                let source = input
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let source_lines: Vec<Value> = source
                    .lines()
                    .enumerate()
                    .map(|(i, line)| {
                        let total = source.lines().count();
                        if i < total - 1 {
                            Value::String(format!("{line}\n"))
                        } else {
                            Value::String(line.to_string())
                        }
                    })
                    .collect();

                let source_lines = if source_lines.is_empty() {
                    vec![Value::String(String::new())]
                } else {
                    source_lines
                };

                let new_cell = if cell_type == "code" {
                    json!({
                        "cell_type": "code",
                        "execution_count": null,
                        "metadata": {},
                        "outputs": [],
                        "source": source_lines
                    })
                } else {
                    json!({
                        "cell_type": "markdown",
                        "metadata": {},
                        "source": source_lines
                    })
                };

                let idx = cell_index.min(cells.len());
                cells.insert(idx, new_cell);
                format!("Inserted {cell_type} cell at index {idx}.")
            }
            "replace_cell" => {
                if cell_index >= cells.len() {
                    return Err(AppError::Tool(format!(
                        "cell_index {cell_index} out of range (notebook has {} cells)",
                        cells.len()
                    )));
                }

                let source = input
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let source_lines: Vec<Value> = source
                    .lines()
                    .enumerate()
                    .map(|(i, line)| {
                        let total = source.lines().count();
                        if i < total - 1 {
                            Value::String(format!("{line}\n"))
                        } else {
                            Value::String(line.to_string())
                        }
                    })
                    .collect();

                let source_lines = if source_lines.is_empty() {
                    vec![Value::String(String::new())]
                } else {
                    source_lines
                };

                cells[cell_index]["source"] = Value::Array(source_lines);
                format!("Replaced source of cell at index {cell_index}.")
            }
            "delete_cell" => {
                if cell_index >= cells.len() {
                    return Err(AppError::Tool(format!(
                        "cell_index {cell_index} out of range (notebook has {} cells)",
                        cells.len()
                    )));
                }

                cells.remove(cell_index);
                format!("Deleted cell at index {cell_index}.")
            }
            _ => {
                return Err(AppError::Tool(format!(
                    "unknown operation '{operation}': expected insert_cell, replace_cell, or delete_cell"
                )));
            }
        };

        let output = serde_json::to_string_pretty(&notebook)
            .map_err(|e| AppError::Tool(format!("failed to serialize notebook: {e}")))?;

        tokio::fs::write(notebook_path, output)
            .await
            .map_err(|e| AppError::Tool(format!("cannot write '{notebook_path}': {e}")))?;

        Ok(result)
    }
}
