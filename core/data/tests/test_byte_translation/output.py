from __future__ import annotations

from pydantic import BaseModel, BeforeValidator, ConfigDict, Field, PlainSerializer
from typing import Annotated


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

    this_is_bits: Annotated[bytes, BeforeValidator(deserialize_binary_data), PlainSerializer(serialize_binary_data)] = Field(alias="thisIsBits")
    this_is_redundant: Annotated[bytes, BeforeValidator(deserialize_binary_data), PlainSerializer(serialize_binary_data)] = Field(alias="thisIsRedundant")

