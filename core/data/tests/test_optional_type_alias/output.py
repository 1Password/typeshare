"""
 Generated by typeshare 1.12.0
"""
from __future__ import annotations

from pydantic import BaseModel


OptionalU16 = int


OptionalU32 = int


class FooBar(BaseModel):
    foo: OptionalU32
    bar: OptionalU16

