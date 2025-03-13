from __future__ import annotations

from pydantic import BaseModel, Field
from typing import Dict, List, Optional


class QualifiedTypes(BaseModel):
    unqualified: str
    qualified: str
    qualified_vec: List[str]
    qualified_hashmap: Dict[str, str]
    qualified_optional: Optional[str] = Field(default=None)
    qualfied_optional_hashmap_vec: Optional[Dict[str, List[str]]] = Field(default=None)

