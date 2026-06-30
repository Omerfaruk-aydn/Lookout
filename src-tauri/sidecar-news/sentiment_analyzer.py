import json
import os
import sys
import traceback
from typing import List, Literal, Optional

from openai import OpenAI
from pydantic import BaseModel, Field


class NewsSentimentItem(BaseModel):
    headline: str
    source: str
    published_at: str
    sentiment: Literal["positive", "negative", "neutral"]
    impact_score: int = Field(ge=0, le=10)
    reasoning: str = Field(max_length=100)


class SentimentBatchRequest(BaseModel):
    headlines: List[dict]
    model: str = "anthropic/claude-haiku-3.5"


class SentimentBatchResponse(BaseModel):
    success: bool
    items: List[NewsSentimentItem] = Field(default_factory=list)
    error: Optional[str] = None


def analyze_batch(request: SentimentBatchRequest) -> SentimentBatchResponse:
    if not request.headlines:
        return SentimentBatchResponse(success=True, items=[])

    try:
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            raise RuntimeError("OPENROUTER_API_KEY not set")

        client = OpenAI(
            base_url="https://openrouter.ai/api/v1",
            api_key=api_key,
        )

        numbered_list = "\n".join(
            f"{i+1}. [{h['source']}] {h['headline']}"
            for i, h in enumerate(request.headlines)
        )

        system_prompt = """You are a financial news sentiment analyzer. Return ONLY a JSON array of objects with fields:
- "headline": original headline
- "source": news source
- "published_at": publication date
- "sentiment": "positive" | "negative" | "neutral"
- "impact_score": 0-10 integer
- "reasoning": max 100 characters

No markdown, no explanation, just the JSON array."""

        response = client.chat.completions.create(
            model=request.model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": f"Analyze:\n\n{numbered_list}"},
            ],
            max_tokens=4096,
            temperature=0.1,
            timeout=15,
        )

        content = response.choices[0].message.content
        if content is None:
            raise RuntimeError("Empty response")

        content = content.strip()
        if content.startswith("```"):
            lines = content.split("\n")
            content = "\n".join(lines[1:-1])

        raw_items = json.loads(content)
        items = [NewsSentimentItem(**item) for item in raw_items]

        return SentimentBatchResponse(success=True, items=items)
    except Exception as e:
        return SentimentBatchResponse(success=False, error=str(e))


def main() -> None:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = SentimentBatchRequest.model_validate_json(line)
            response = analyze_batch(request)
            print(response.model_dump_json(), flush=True)
        except Exception:
            error_response = SentimentBatchResponse(
                success=False, error=traceback.format_exc()
            )
            print(error_response.model_dump_json(), flush=True)


if __name__ == "__main__":
    main()
