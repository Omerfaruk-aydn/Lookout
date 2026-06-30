from pydantic import BaseModel, Field
from typing import List, Optional


class SupportResistanceEstimate(BaseModel):
    support: List[float] = Field(default_factory=list)
    resistance: List[float] = Field(default_factory=list)


class IndicatorObservation(BaseModel):
    name: str
    value_estimate: str


class VisionData(BaseModel):
    ticker_visible: Optional[str] = None
    trend_direction: Optional[str] = None
    visible_patterns: List[str] = Field(default_factory=list)
    support_resistance_estimate: SupportResistanceEstimate = Field(
        default_factory=SupportResistanceEstimate
    )
    volume_observation: Optional[str] = None
    indicators_visible: List[IndicatorObservation] = Field(default_factory=list)
    confidence: float = Field(ge=0.0, le=1.0)
    notes: Optional[str] = None


class VisionRequest(BaseModel):
    image_base64: str
    request_id: str
    model: str = "anthropic/claude-sonnet-4-6"


class VisionResponse(BaseModel):
    request_id: str
    success: bool
    data: Optional[VisionData] = None
    error: Optional[str] = None
