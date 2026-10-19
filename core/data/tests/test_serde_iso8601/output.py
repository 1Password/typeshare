from __future__ import annotations

from datetime import datetime
from pydantic import BaseModel, BeforeValidator, PlainSerializer
from typing import Annotated


def serialize_datetime_data(utc_time: datetime) -> str:
        return utc_time.strftime("%Y-%m-%dT%H:%M:%S.%fZ")

def parse_rfc3339(date_str: str) -> datetime:
    date_formats = [
        "%Y-%m-%dT%H:%M:%SZ",   
        "%Y-%m-%dT%H:%M:%S.%fZ"
    ]

    # %f accepts at most 6 fractional digits, but Rust can emit up to 9 (nanoseconds)
    head, dot, fraction = date_str.partition(".")
    if dot and fraction.endswith("Z"):
        date_str = f"{head}.{fraction[:-1][:6]}Z"
    
    for fmt in date_formats:
        try:
            return datetime.strptime(date_str, fmt)
        except ValueError:
            continue
    
    raise ValueError(f"Invalid RFC 3339 date format: {date_str}")

class Foo(BaseModel):
    time: Annotated[datetime, BeforeValidator(parse_rfc3339), PlainSerializer(serialize_datetime_data)]

