from __future__ import annotations

from enum import Enum
from pydantic import BaseModel
from typing import Literal, Union


class SomeEnumTypes(str, Enum):
    CONTEXT = "Context"
    OTHER = "Other"

class SomeEnumContext(BaseModel):
    """
    The associated String contains some opaque context
    """
    type: Literal[SomeEnumTypes.CONTEXT] = SomeEnumTypes.CONTEXT
    content: str

class SomeEnumOther(BaseModel):
    type: Literal[SomeEnumTypes.OTHER] = SomeEnumTypes.OTHER
    content: int

SomeEnum = Union[SomeEnumContext, SomeEnumOther]
