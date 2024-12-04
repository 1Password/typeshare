from __future__ import annotations

from pydantic import BaseModel


Bar = str

class Foo(BaseModel):
    bar: Bar

