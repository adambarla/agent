import os
from dotenv import load_dotenv
from openai import OpenAI

load_dotenv()

def main() -> None:
    
  client = OpenAI(
      api_key=os.getenv('DEEPSEEK_API_KEY'),
      base_url="https://api.deepseek.com")
  
  response = client.chat.completions.create(
      model="deepseek-v4-pro",
      messages=[
          {"role": "system", "content": "You are a helpful assistant"},
          {"role": "user", "content": "Hello"},
      ],
      stream=False,
      reasoning_effort="high",
      extra_body={"thinking": {"type": "enabled"}}
  )
  
  print(response.choices[0].message.content)

if __name__ == "__main__":
    main()