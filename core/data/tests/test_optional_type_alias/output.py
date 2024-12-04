from __future__ import annotations

from pydantic import BaseModel
from typing import Optional


OptionalU16 = Optional[int]

OptionalU32 = Optional[int]

class FooBar(BaseModel):
    foo: OptionalU32
    bar: OptionalU16

