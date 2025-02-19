from __future__ import annotations

from datetime import datetime
from pydantic import BaseModel


class Foo(BaseModel):
    time: datetime

