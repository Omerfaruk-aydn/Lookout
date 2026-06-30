import json
import os
import sys
import traceback
from datetime import datetime, timedelta, timezone
from typing import List, Literal, Optional

import requests
from openai import OpenAI
from pydantic import BaseModel, Field


class NewsItem(BaseModel):
    headline: str
    source: str
    url: str
    published_at: str
    summary: Optional[str] = None


class NewsSentimentItem(BaseModel):
    headline: str
    source: str
    published_at: str
    sentiment: Literal["positive", "negative", "neutral"]
    impact_score: int = Field(ge=0, le=10)
    reasoning: str = Field(max_length=100)


class AggregatedSentiment(BaseModel):
    ticker: str
    overall_sentiment: Literal["positive", "negative", "neutral", "mixed"]
    weighted_score: float = Field(ge=-1.0, le=1.0)
    item_count: int
    items: List[NewsSentimentItem] = Field(default_factory=list)


class NewsRequest(BaseModel):
    ticker: str
    hours_back: int = 48
    request_id: str
    model: str = "anthropic/claude-haiku-3.5"


class NewsResponse(BaseModel):
    request_id: str
    success: bool
    data: Optional[AggregatedSentiment] = None
    error: Optional[str] = None


def fetch_news(ticker: str, hours_back: int) -> List[NewsItem]:
    api_key = os.environ.get("FINNHUB_API_KEY")
    if not api_key:
        raise RuntimeError("FINNHUB_API_KEY environment variable is not set")

    now = datetime.now(timezone.utc)
    from_date = now - timedelta(hours=hours_back)

    from_str = from_date.strftime("%Y-%m-%d")
    to_str = now.strftime("%Y-%m-%d")

    url = f"https://finnhub.io/api/v1/company-news?symbol={ticker}&from={from_str}&to={to_str}&token={api_key}"

    response = requests.get(url, timeout=10)
    response.raise_for_status()

    raw_items = response.json()
    cutoff = now - timedelta(hours=hours_back)

    news_items = []
    for item in raw_items:
        pub_ts = item.get("datetime", 0)
        pub_dt = datetime.fromtimestamp(pub_ts, tz=timezone.utc)
        if pub_dt < cutoff:
            continue
        news_items.append(
            NewsItem(
                headline=item.get("headline", ""),
                source=item.get("source", ""),
                url=item.get("url", ""),
                published_at=pub_dt.isoformat(),
                summary=item.get("summary", ""),
            )
        )

    return news_items


def analyze_sentiment_batch(
    client: OpenAI, news_items: List[NewsItem], model: str
) -> List[NewsSentimentItem]:
    if not news_items:
        return []

    numbered_list = "\n".join(
        f"{i+1}. [{item.source}] {item.headline}" for i, item in enumerate(news_items)
    )

    system_prompt = """You are a financial news sentiment analyzer. You will receive a numbered list of news headlines.
For each headline, return a JSON object with:
- "headline": the original headline
- "source": the news source
- "sentiment": one of "positive", "negative", "neutral"
- "impact_score": integer 0-10 indicating potential market impact
- "reasoning": brief reasoning (max 100 characters)

Return ONLY a JSON array of objects. No markdown, no explanation."""

    user_prompt = f"Analyze the sentiment of these news headlines:\n\n{numbered_list}"

    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ],
        max_tokens=4096,
        temperature=0.1,
        timeout=15,
    )

    content = response.choices[0].message.content
    if content is None:
        raise RuntimeError("Empty response from sentiment API")

    content = content.strip()
    if content.startswith("```"):
        lines = content.split("\n")
        content = "\n".join(lines[1:-1])

    raw_items = json.loads(content)
    results = []
    for raw in raw_items:
        results.append(NewsSentimentItem(**raw))

    return results


def aggregate_sentiment(
    ticker: str, items: List[NewsSentimentItem]
) -> AggregatedSentiment:
    if not items:
        return AggregatedSentiment(
            ticker=ticker,
            overall_sentiment="neutral",
            weighted_score=0.0,
            item_count=0,
            items=[],
        )

    sentiment_map = {"positive": 1.0, "negative": -1.0, "neutral": 0.0}

    total_weight = 0.0
    weighted_sum = 0.0

    for i, item in enumerate(items):
        recency_weight = 2.0 if i < len(items) // 2 else 1.0
        impact_weight = item.impact_score / 10.0
        weight = recency_weight * (1.0 + impact_weight)
        weighted_sum += sentiment_map.get(item.sentiment, 0.0) * weight
        total_weight += weight

    score = weighted_sum / total_weight if total_weight > 0 else 0.0

    if score > 0.2:
        overall = "positive"
    elif score < -0.2:
        overall = "negative"
    elif abs(score) <= 0.1:
        overall = "neutral"
    else:
        overall = "mixed"

    return AggregatedSentiment(
        ticker=ticker,
        overall_sentiment=overall,
        weighted_score=round(score, 4),
        item_count=len(items),
        items=items,
    )


def process_request(request: NewsRequest) -> NewsResponse:
    try:
        news_items = fetch_news(request.ticker, request.hours_back)
    except Exception as e:
        return NewsResponse(
            request_id=request.request_id,
            success=False,
            error=f"Failed to fetch news: {e}",
        )

    if not news_items:
        empty_sentiment = AggregatedSentiment(
            ticker=request.ticker,
            overall_sentiment="neutral",
            weighted_score=0.0,
            item_count=0,
            items=[],
        )
        return NewsResponse(
            request_id=request.request_id,
            success=True,
            data=empty_sentiment,
        )

    try:
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            raise RuntimeError("OPENROUTER_API_KEY environment variable is not set")

        client = OpenAI(
            base_url="https://openrouter.ai/api/v1",
            api_key=api_key,
        )

        sentiment_items = analyze_sentiment_batch(client, news_items, request.model)
        aggregated = aggregate_sentiment(request.ticker, sentiment_items)

        return NewsResponse(
            request_id=request.request_id,
            success=True,
            data=aggregated,
        )
    except Exception as e:
        return NewsResponse(
            request_id=request.request_id,
            success=False,
            error=f"Sentiment analysis failed: {e}",
        )


def main() -> None:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = NewsRequest.model_validate_json(line)
            response = process_request(request)
            print(response.model_dump_json(), flush=True)
        except Exception:
            error_response = NewsResponse(
                request_id="unknown",
                success=False,
                error=f"Unexpected error: {traceback.format_exc()}",
            )
            print(error_response.model_dump_json(), flush=True)


if __name__ == "__main__":
    main()
