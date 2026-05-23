import argparse
import logging
import os
from datetime import datetime
from typing import Optional, cast

from dotenv import load_dotenv
from openai import OpenAI
from openai.types.chat import (
  ChatCompletionMessage,
  ChatCompletionMessageParam,
)
from openai.types.shared.reasoning_effort import ReasoningEffort

from agent.data.prompts import SYSTEM_PROMPT
from agent.tools import AddTool, ToolRegistry

load_dotenv()

logger = logging.getLogger(__name__)

client = OpenAI(
  api_key=os.getenv("DEEPSEEK_API_KEY"), base_url="https://api.deepseek.com"
)


def get_user_input() -> str:
  user_input = ""
  while True:
    user_input += input("You: ")
    if len(user_input) == 0:
      print("Input cannot be empty")
      continue
    if user_input[0] == "/":
      command = user_input[1:].strip()
      match command:
        case "exit":
          raise SystemExit
        case _:
          print(f"Unknown command: {command}")
          continue
    else:
      break
  return user_input


def call(
  messages: list[ChatCompletionMessageParam],
  tool_registry: ToolRegistry,
  client: OpenAI = client,
  model: str = "deepseek-v4-flash",
  reasoning_effort: Optional[ReasoningEffort] = "low",
  thinking: bool = True,
) -> ChatCompletionMessage:
  response = client.chat.completions.create(
    model=model,
    stream=False,
    messages=messages,
    reasoning_effort=reasoning_effort,
    extra_body={"thinking": {"type": "enabled" if thinking else "disabled"}},
    tools=tool_registry.get_tools(),
  )
  logger.debug(response)
  return response.choices[0].message


def chat_loop(
  messages: list[ChatCompletionMessageParam],
  tool_registry: ToolRegistry = ToolRegistry(),
) -> None:
  while True:
    user_input = get_user_input()

    logger.info(f"User input: {user_input}")

    messages.append({"role": "user", "content": user_input})
    while True:
      message = call(messages, tool_registry=tool_registry)
      messages.append(cast(ChatCompletionMessageParam, message))

      if message.content:
        logger.info(f"Agent response: {message.content}")
        print(f"Agent: {message.content}")

      if not message.tool_calls:
        break

      messages.extend(tool_registry.call_tools(message.tool_calls))

    logger.debug(messages)


def main() -> None:
  parser = argparse.ArgumentParser(description="Agent")
  parser.add_argument("--log", default="info", help="Log level")
  args = parser.parse_args()
  date_str = datetime.now().strftime("%Y%m%d")
  time_str = datetime.now().strftime("%H%M%S")
  log_dir = f"logs/{date_str}/"
  os.makedirs(log_dir, exist_ok=True)
  logging.basicConfig(level=args.log.upper(), filename=f"{log_dir}chat_{time_str}.log")

  logger.info("Starting agent")

  messages: list[ChatCompletionMessageParam] = [
    {"role": "system", "content": SYSTEM_PROMPT},
  ]
  tool_registry = ToolRegistry()
  tool_registry.register(AddTool())
  try:
    chat_loop(messages, tool_registry=tool_registry)
  except (KeyboardInterrupt, SystemExit):
    logger.info("Chat ended")


if __name__ == "__main__":
  main()
