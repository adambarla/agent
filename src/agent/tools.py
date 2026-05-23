import json
import logging
from typing import Iterable, Optional, Protocol, cast

from openai.types.chat import (
  ChatCompletionMessageFunctionToolCall,
  ChatCompletionMessageToolCallUnion,
  ChatCompletionToolMessageParam,
  ChatCompletionToolUnionParam,
)

logger = logging.getLogger(__name__)


class Tool(Protocol):
  _description: str
  _name: str

  def call(self, *args, **kwargs) -> str: ...

  def to_dict(self) -> ChatCompletionToolUnionParam: ...


class ToolRegistry:
  def __init__(self):
    self._tools: dict[str, Tool] = {}

  def register(self, tool: Tool):
    self._tools[tool._name] = tool

  def get(self, name: str) -> Optional[Tool]:
    if name in self._tools:
      return self._tools[name]
    return None

  def get_tools(self) -> list[ChatCompletionToolUnionParam]:
    return [tool.to_dict() for tool in self._tools.values()]

  def call_tools(
    self, tool_calls: Iterable[ChatCompletionMessageToolCallUnion]
  ) -> list[ChatCompletionToolMessageParam]:
    results: list[ChatCompletionToolMessageParam] = []
    for tool_call in tool_calls:
      if tool_call.type != "function":
        continue
      function_tool_call = cast(ChatCompletionMessageFunctionToolCall, tool_call)

      tool = self.get(function_tool_call.function.name)
      if tool is None:
        logger.error(f'Tool "{function_tool_call.function.name}" not found')
        continue
      arguments = json.loads(function_tool_call.function.arguments)
      result = tool.call(**arguments)
      logger.info(
        'Tool "%s" with arguments %s returned "%s"',
        function_tool_call.function.name,
        arguments,
        result,
      )

      results.append(
        {"role": "tool", "tool_call_id": function_tool_call.id, "content": result}
      )
    return results


class AddTool:
  _description: str = "Add two numbers"
  _name: str = "add"

  def call(self, a: float, b: float) -> str:
    return str(a + b)

  def to_dict(self) -> ChatCompletionToolUnionParam:
    return {
      "type": "function",
      "function": {
        "name": self._name,
        "description": self._description,
        "parameters": {
          "type": "object",
          "properties": {"a": {"type": "number"}, "b": {"type": "number"}},
          "required": ["a", "b"],
        },
      },
    }
