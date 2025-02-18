from __future__ import annotations

from pydantic import BaseModel, field_validator
from typing import Any


class Foo(BaseModel):
    bytes: bytes

    @field_validator('content', mode='before')
    @classmethod
    def cast_list_to_bytes(cls, value: Any) -> bytes:
        if isinstance(value, list) and all(isinstance(i, int) for i in value):
          return bytes(value)

        raise ValueError("content must be a list of integers")

    class Config:
        json_encoders = {
            bytes: lambda b: list(b),
        }
