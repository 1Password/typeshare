from __future__ import annotations

from pydantic import BaseModel


class Foo(BaseModel):
    """
    This is a comment.
    """
    a: int
    b: int
    c: int
    e: int
    f: int
    g: int

