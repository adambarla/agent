use anyhow::{Context, Result, bail};
use json::{JsonValue, object};
use std::collections::HashMap;

pub trait Tool {
    fn name(&self) -> &str;
    fn schema(&self) -> JsonValue;
    fn call(&self, arguments: &JsonValue) -> Result<String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        log::info!("registering tool: {name}");
        self.tools.insert(name, Box::new(tool));
    }

    pub fn schemas(&self) -> Result<JsonValue> {
        let mut schemas = JsonValue::new_array();

        for tool in self.tools.values() {
            log::debug!("exposing tool schema: {}", tool.name());
            schemas
                .push(tool.schema())
                .context("failed to add tool schema")?;
        }

        log::info!("exposed {} tool schema(s)", schemas.len());
        Ok(schemas)
    }

    pub fn call_tools(&self, tool_calls: &JsonValue) -> Result<Vec<JsonValue>> {
        let mut messages = Vec::new();
        log::info!("processing {} tool call(s)", tool_calls.len());

        for tool_call in tool_calls.members() {
            if tool_call["type"].as_str() != Some("function") {
                log::warn!(
                    "skipping tool with unsupported tool call type: {}",
                    tool_call["type"].as_str().unwrap_or("unknown")
                );
                continue;
            }

            let id = tool_call["id"].as_str().context("tool call missing id")?;
            let name = tool_call["function"]["name"]
                .as_str()
                .context("tool call missing function name")?;
            let arguments_text = tool_call["function"]["arguments"]
                .as_str()
                .context("tool call missing function arguments")?;
            let arguments =
                json::parse(arguments_text).context("failed to parse tool arguments")?;

            log::debug!("tool call id: {id}");

            let result = match self.tools.get(name) {
                Some(tool) => match tool.call(&arguments) {
                    Ok(result) => {
                        log::info!(
                            "tool \"{name}\" called with arguments {} returned {result}",
                            arguments.dump()
                        );
                        result
                    }
                    Err(error) => {
                        log::error!(
                            "tool \"{name}\" called with arguments {} failed: {error:?}",
                            arguments.dump()
                        );
                        format!("Tool `{name}` failed: {error:#}")
                    }
                },
                None => {
                    log::error!("tool not found: {name}");
                    format!("Tool `{name}` is not available")
                }
            };

            let mut message = JsonValue::new_object();
            message["role"] = "tool".into();
            message["tool_call_id"] = id.into();
            message["content"] = result.into();
            messages.push(message);
        }

        Ok(messages)
    }
}

pub struct AddTool;

impl Tool for AddTool {
    fn name(&self) -> &str {
        "add"
    }

    fn schema(&self) -> JsonValue {
        object! {
            type: "function",
            function: {
                name: "add",
                description: "Add two numbers",
                parameters: {
                    type: "object",
                    properties: {
                        a: { type: "number" },
                        b: { type: "number" },
                    },
                    required: ["a", "b"],
                },
            },
        }
    }

    fn call(&self, arguments: &JsonValue) -> Result<String> {
        let a = arguments["a"].as_f64().context("missing argument a")?;
        let b = arguments["b"].as_f64().context("missing argument b")?;

        if !a.is_finite() || !b.is_finite() {
            bail!("arguments must be finite numbers");
        }

        Ok((a + b).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn function_tool_call(id: &str, name: &str, arguments: &str) -> JsonValue {
        let mut function = JsonValue::new_object();
        function["name"] = name.into();
        function["arguments"] = arguments.into();

        let mut tool_call = JsonValue::new_object();
        tool_call["id"] = id.into();
        tool_call["type"] = "function".into();
        tool_call["function"] = function;

        tool_call
    }

    #[test]
    fn add_tool_adds_two_numbers() -> Result<()> {
        let arguments = json::parse(r#"{"a":2,"b":3.5}"#)?;
        let result = AddTool.call(&arguments)?;

        assert_eq!(result, "5.5");
        Ok(())
    }

    #[test]
    fn registry_returns_tool_message_for_successful_call() -> Result<()> {
        let mut registry = ToolRegistry::new();
        registry.register(AddTool);

        let mut tool_calls = JsonValue::new_array();
        tool_calls.push(function_tool_call("call_1", "add", r#"{"a":2,"b":3}"#))?;

        let messages = registry.call_tools(&tool_calls)?;

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "tool");
        assert_eq!(messages[0]["tool_call_id"], "call_1");
        assert_eq!(messages[0]["content"], "5");

        Ok(())
    }

    #[test]
    fn registry_returns_tool_message_for_tool_error() -> Result<()> {
        let mut registry = ToolRegistry::new();
        registry.register(AddTool);

        let mut tool_calls = JsonValue::new_array();
        tool_calls.push(function_tool_call("call_1", "add", r#"{"a":2}"#))?;

        let messages = registry.call_tools(&tool_calls)?;

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "tool");
        assert_eq!(messages[0]["tool_call_id"], "call_1");

        let content = messages[0]["content"].as_str().context("missing content")?;
        assert!(content.contains("Tool `add` failed"));
        assert!(content.contains("missing argument b"));

        Ok(())
    }

    #[test]
    fn registry_returns_tool_message_for_unknown_tool() -> Result<()> {
        let registry = ToolRegistry::new();

        let mut tool_calls = JsonValue::new_array();
        tool_calls.push(function_tool_call("call_1", "missing", r#"{}"#))?;

        let messages = registry.call_tools(&tool_calls)?;

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "tool");
        assert_eq!(messages[0]["tool_call_id"], "call_1");
        assert_eq!(messages[0]["content"], "Tool `missing` is not available");

        Ok(())
    }

    #[test]
    fn registry_errors_for_malformed_tool_call() -> Result<()> {
        let registry = ToolRegistry::new();

        let mut tool_call = JsonValue::new_object();
        tool_call["type"] = "function".into();

        let mut tool_calls = JsonValue::new_array();
        tool_calls.push(tool_call)?;

        let error = registry
            .call_tools(&tool_calls)
            .expect_err("missing id should be a protocol error");

        assert!(format!("{error:#}").contains("tool call missing id"));

        Ok(())
    }
}
