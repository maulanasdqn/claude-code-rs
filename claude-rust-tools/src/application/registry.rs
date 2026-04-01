use std::collections::HashMap;
use std::sync::Arc;

use claude_rust_types::Tool;
use serde_json::{Value, json};

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn tool_definitions(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|t| {
                json!({
                    "name": t.name(),
                    "description": t.description(),
                    "input_schema": t.input_schema(),
                })
            })
            .collect()
    }

    /// Return tool definitions filtered by a predicate on the tool trait.
    pub fn tool_definitions_filtered<F>(&self, predicate: F) -> Vec<Value>
    where
        F: Fn(&dyn Tool) -> bool,
    {
        self.tools
            .values()
            .filter(|t| predicate(t.as_ref()))
            .map(|t| {
                json!({
                    "name": t.name(),
                    "description": t.description(),
                    "input_schema": t.input_schema(),
                })
            })
            .collect()
    }

    pub fn tool_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tools.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn clone_excluding(&self, exclude: &[&str]) -> Self {
        let tools = self
            .tools
            .iter()
            .filter(|(name, _)| !exclude.contains(&name.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self { tools }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
