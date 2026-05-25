use anyhow::{Context, Result};
use json::JsonValue;

use crate::constants::SYSTEM_PROMPT;
use crate::llm;
use crate::tools::{AddTool, ToolRegistry};
use crate::utils;

pub fn run() -> Result<()> {
    dotenv::dotenv().ok();
    utils::init_logging()?;
    log::info!("agent started");

    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(AddTool);
    let tool_schemas = tool_registry.schemas()?;

    let mut messages = JsonValue::new_array();
    let mut system_message = JsonValue::new_object();
    system_message["role"] = "system".into();
    system_message["content"] = SYSTEM_PROMPT.into();
    messages
        .push(system_message)
        .context("failed to add system message")?;

    agent_loop(&mut messages, &tool_schemas, &tool_registry)?;

    Ok(())
}

fn agent_loop(
    messages: &mut JsonValue,
    tools: &JsonValue,
    tool_registry: &ToolRegistry,
) -> Result<()> {
    loop {
        let user_input = utils::get_input()?;
        if user_input == "exit" {
            break;
        }

        let mut user_message = JsonValue::new_object();
        user_message["role"] = "user".into();
        user_message["content"] = user_input.into();
        messages
            .push(user_message)
            .context("failed to add user message")?;

        loop {
            let body = build_chat_request(messages, tools);
            let response = llm::call(body)?;

            let assistant_message = response["choices"][0]["message"].clone();
            let tool_calls = assistant_message["tool_calls"].clone();

            messages
                .push(assistant_message.clone())
                .context("failed to add assistant message")?;

            let content = assistant_message["content"]
                .as_str()
                .context("response did not contain choices[0].message.content")?;

            if !content.is_empty() {
                println!("Assistant: {content}");
            }

            if tool_calls.is_empty() {
                break;
            }

            for tool_message in tool_registry.call_tools(&tool_calls)? {
                messages
                    .push(tool_message)
                    .context("failed to add tool message")?;
            }
        }
    }
    Ok(())
}

fn build_chat_request(messages: &JsonValue, tools: &JsonValue) -> String {
    format!(
        r#"{{"model":"deepseek-v4-flash","messages":{},"tools":{}}}"#,
        messages.dump(),
        tools.dump()
    )
}
