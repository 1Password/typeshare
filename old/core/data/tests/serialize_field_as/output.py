from __future__ import annotations

from pydantic import BaseModel, Field
from typing import List, Optional


class EditItemViewModelSaveRequest(BaseModel):
    context: str
    values: List[EditItemSaveValue]
    fill_action: Optional[AutoFillItemActionRequest] = Field(default=None)

