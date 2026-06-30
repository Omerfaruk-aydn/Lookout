import json
import os
import sys
import traceback
from typing import Optional

from openai import OpenAI
from pydantic import ValidationError

from prompts import SYSTEM_PROMPT, RETRY_PROMPT
from schema import VisionData, VisionRequest, VisionResponse


def get_client() -> OpenAI:
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        raise RuntimeError("OPENROUTER_API_KEY environment variable is not set")
    return OpenAI(
        base_url="https://openrouter.ai/api/v1",
        api_key=api_key,
    )


def call_vision_api(
    client: OpenAI,
    image_base64: str,
    model: str,
    system_prompt: str,
) -> str:
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": system_prompt},
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": f"data:image/png;base64,{image_base64}",
                        },
                    },
                    {
                        "type": "text",
                        "text": "Analyze this chart screenshot and return your analysis as JSON.",
                    },
                ],
            },
        ],
        max_tokens=2048,
        temperature=0.2,
        timeout=15,
    )
    content = response.choices[0].message.content
    if content is None:
        raise RuntimeError("Empty response from vision API")
    return content.strip()


def process_request(request: VisionRequest) -> VisionResponse:
    try:
        client = get_client()
    except RuntimeError as e:
        return VisionResponse(
            request_id=request.request_id,
            success=False,
            error=str(e),
        )

    raw_response: Optional[str] = None
    try:
        raw_response = call_vision_api(
            client, request.image_base64, request.model, SYSTEM_PROMPT
        )
        data = VisionData.model_validate_json(raw_response)
        return VisionResponse(
            request_id=request.request_id,
            success=True,
            data=data,
        )
    except (ValidationError, json.JSONDecodeError):
        try:
            raw_response = call_vision_api(
                client, request.image_base64, request.model, RETRY_PROMPT
            )
            data = VisionData.model_validate_json(raw_response)
            return VisionResponse(
                request_id=request.request_id,
                success=True,
                data=data,
            )
        except (ValidationError, json.JSONDecodeError) as e:
            return VisionResponse(
                request_id=request.request_id,
                success=False,
                error=f"Schema validation failed after retry: {e}",
            )
        except Exception as e:
            return VisionResponse(
                request_id=request.request_id,
                success=False,
                error=f"Retry API call failed: {e}",
            )
    except Exception as e:
        return VisionResponse(
            request_id=request.request_id,
            success=False,
            error=f"Vision API call failed: {e}",
        )


def main() -> None:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = VisionRequest.model_validate_json(line)
            response = process_request(request)
            print(response.model_dump_json(), flush=True)
        except ValidationError as e:
            error_response = VisionResponse(
                request_id="unknown",
                success=False,
                error=f"Invalid request: {e}",
            )
            print(error_response.model_dump_json(), flush=True)
        except Exception:
            error_response = VisionResponse(
                request_id="unknown",
                success=False,
                error=f"Unexpected error: {traceback.format_exc()}",
            )
            print(error_response.model_dump_json(), flush=True)


if __name__ == "__main__":
    main()
