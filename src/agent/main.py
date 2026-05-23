import argparse
import logging
import os
from datetime import datetime

from dotenv import load_dotenv
from openai import OpenAI
from openai.types.chat import ChatCompletionMessageParam

from agent.data.prompts import SYSTEM_PROMPT

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


def chat_loop(messages: list[ChatCompletionMessageParam]) -> None:
  while True:
    user_input = get_user_input()

    logger.info(f"User input: {user_input}")

    messages.append({"role": "user", "content": user_input})
    response = client.chat.completions.create(
      model="deepseek-v4-flash",
      stream=False,
      messages=messages,
      reasoning_effort="low",
      extra_body={"thinking": {"type": "enabled"}},
    )
    assistant_content = response.choices[0].message.content or ""
    messages.append({"role": "assistant", "content": assistant_content})
    print(f"Agent: {assistant_content}")

    logger.info(f"Agent response: {assistant_content}")
    logging.debug(response)
    logging.debug(messages)


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
  try:
    chat_loop(messages)
  except (KeyboardInterrupt, SystemExit):
    logger.info("Chat ended")


if __name__ == "__main__":
  main()
