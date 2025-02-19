from __future__ import annotations

from pydantic import BaseModel, field_serializer, field_validator


class Foo(BaseModel):
    bytes: bytes

    
    @field_serializer("content")
    def serialize_data(self, value: bytes) -> list[int]:
        return list(value)
                
    @field_validator("content", mode="before")
    def deserialize_data(cls, value):
        if isinstance(value, list): 
            return bytes(value)
        return value
