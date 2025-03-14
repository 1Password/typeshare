from __future__ import annotations

from datetime import datetime
from pydantic import BaseModel, BeforeValidator, ConfigDict, Field, PlainSerializer
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

def serialize_binary_data(value: bytes) -> list[int]:
        return list(value)

def deserialize_binary_data(value):
     if isinstance(value, list):
         if all(isinstance(x, int) and 0 <= x <= 255 for x in value):
            return bytes(value)
         raise ValueError("All elements must be integers in the range 0-255 (u8).")
     elif isinstance(value, bytes):
            return value
     raise TypeError("Content must be a list of integers (0-255) or bytes.")

class Foo(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    time: Annotated[datetime, BeforeValidator(parse_rfc3339), PlainSerializer(serialize_datetime_data)]
    time_2: Annotated[datetime, BeforeValidator(parse_rfc3339), PlainSerializer(serialize_datetime_data)] = Field(alias="time2")
    time_3: Annotated[datetime, BeforeValidator(parse_rfc3339), PlainSerializer(serialize_datetime_data)] = Field(alias="time3")
    bytes: Annotated[bytes, BeforeValidator(deserialize_binary_data), PlainSerializer(serialize_binary_data)]
    bytes_2: Annotated[bytes, BeforeValidator(deserialize_binary_data), PlainSerializer(serialize_binary_data)] = Field(alias="bytes2")

