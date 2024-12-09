from __future__ import annotations

from pydantic import BaseModel
from typing import List


class Video(BaseModel):
    tags: List[Tag]

