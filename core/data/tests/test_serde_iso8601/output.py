"""
 Generated by typeshare 1.0.0
"""
from __future__ import annotations

from pydantic import BaseModel
from datetime import datetime


class Foo(BaseModel):
    time: DateTime[Utc]


