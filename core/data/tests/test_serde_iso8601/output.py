from __future__ import annotations

from datetime import datetime
from pydantic import BaseModel, BeforeValidator, PlainSerializer
from typing import Annotated


def serialize_datetime_data(utc_time: datetime) -> str:
        return utc_time.strftime("%Y-%m-%dT%H:%M:%SZ")

def parse_rfc3339(date_str: str) -> datetime:
    date_formats = [
        "%Y-%m-%dT%H:%M:%SZ",   
        "%Y-%m-%dT%H:%M:%S.%fZ"
    ]
    
    for fmt in date_formats:
        try:
            return datetime.strptime(date_str, fmt)
        except ValueError:
            continue
    
    raise ValueError(f"Invalid RFC 3339 date format: {date_str}")

class Foo(BaseModel):
    time: Annotated[datetime, BeforeValidator(parse_rfc3339), PlainSerializer(serialize_datetime_data)]

