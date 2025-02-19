from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field, field_serializer, field_validator


class Foo(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    this_is_bits: bytes = Field(alias="thisIsBits")

    
    @field_serializer("thisIsBits")
    def serialize_data(self, value: bytes) -> list[int]:
        return list(value)
                
    @field_validator("thisIsBits", mode="before")
    def deserialize_data(cls, value):
        if isinstance(value, list):
            if all(isinstance(x, int) and 0 <= x <= 255 for x in value): 
                return bytes(value) 
            raise ValueError("All elements must be integers in the range 0-255 (u8).")
        elif isinstance(value, bytes):
            return value
        raise TypeError("thisIsBits must be a list of integers (0-255) or bytes.")
