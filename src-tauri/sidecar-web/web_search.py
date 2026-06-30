import json
import os
import sys
import traceback
from typing import List, Optional

from duckduckgo_search import DDGS
from openai import OpenAI
from pydantic import BaseModel, Field


class SearchResult(BaseModel):
    title: str
    url: str
    snippet: str
    source: str


class WebSearchItem(BaseModel):
    query: str
    results: List[SearchResult] = Field(default_factory=list)


class WebSearchRequest(BaseModel):
    ticker: str
    request_id: str
    max_queries: int = 3
    model: str = "xiaomi/mimo-v2.5"


class WebSearchResponse(BaseModel):
    request_id: str
    success: bool
    data: Optional[dict] = None
    error: Optional[str] = None


def build_queries(ticker: str) -> List[str]:
    return [
        f"{ticker} stock news today",
        f"{ticker} stock analysis forecast",
        f"{ticker} earnings revenue financial",
    ]


def perform_searches(queries: List[str], max_results: int = 5) -> List[WebSearchItem]:
    items = []
    with DDGS() as ddgs:
        for query in queries:
            try:
                results = ddgs.text(query, max_results=max_results)
                search_results = []
                for r in results:
                    search_results.append(
                        SearchResult(
                            title=r.get("title", ""),
                            url=r.get("href", ""),
                            snippet=r.get("body", ""),
                            source=r.get("href", "").split("/")[2]
                            if r.get("href")
                            else "",
                        )
                    )
                items.append(WebSearchItem(query=query, results=search_results))
            except Exception as e:
                items.append(WebSearchItem(query=query, results=[]))
    return items


def summarize_with_llm(
    client: OpenAI, ticker: str, items: List[WebSearchItem], model: str
) -> dict:
    context = f"Web search results for {ticker}:\n\n"
    for item in items:
        context += f"Query: {item.query}\n"
        for r in item.results[:3]:
            context += f"- {r.title}: {r.snippet}\n"
        context += "\n"

    system_prompt = """You are a financial web research assistant. Summarize web search results into a structured JSON response.

Return ONLY valid JSON in this exact format:
{
  "ticker": "...",
  "summary": "2-3 sentences summarizing key findings",
  "key_topics": ["topic1", "topic2"],
  "sentiment": "positive|negative|neutral|mixed",
  "notable_sources": ["source1.com", "source2.com"],
  "confidence": 0.0 to 1.0
}

No markdown, no explanation."""

    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": context},
        ],
        max_tokens=2048,
        temperature=0.3,
        timeout=30,
    )

    content = response.choices[0].message.content
    if content is None:
        raise RuntimeError("Empty response from LLM")

    content = content.strip()
    if content.startswith("```"):
        lines = content.split("\n")
        content = "\n".join(lines[1:-1])

    parsed = json.loads(content)
    return parsed


def process_request(request: WebSearchRequest) -> WebSearchResponse:
    try:
        queries = build_queries(request.ticker)[: request.max_queries]
        items = perform_searches(queries)

        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            raise RuntimeError("OPENROUTER_API_KEY not set")

        client = OpenAI(
            base_url="https://openrouter.ai/api/v1",
            api_key=api_key,
        )

        summary = summarize_with_llm(client, request.ticker, items, request.model)

        return WebSearchResponse(
            request_id=request.request_id,
            success=True,
            data={
                "ticker": request.ticker,
                "summary": summary.get("summary", ""),
                "key_topics": summary.get("key_topics", []),
                "sentiment": summary.get("sentiment", "neutral"),
                "notable_sources": summary.get("notable_sources", []),
                "confidence": summary.get("confidence", 0.0),
                "raw_items": [item.model_dump() for item in items],
            },
        )
    except Exception as e:
        return WebSearchResponse(
            request_id=request.request_id,
            success=False,
            error=f"Web search failed: {e}",
        )


def main() -> None:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = WebSearchRequest.model_validate_json(line)
            response = process_request(request)
            print(response.model_dump_json(), flush=True)
        except Exception:
            error_response = WebSearchResponse(
                request_id="unknown",
                success=False,
                error=f"Unexpected error: {traceback.format_exc()}",
            )
            print(error_response.model_dump_json(), flush=True)


if __name__ == "__main__":
    main()
