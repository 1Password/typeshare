from __future__ import annotations

from enum import Enum
from pydantic import BaseModel, ConfigDict, Field
from typing import Literal


class StructHasVoidType(BaseModel):
    """
    This struct has a unit field
    """
    model_config = ConfigDict(populate_by_name=True)

    this_is_a_unit: None = Field(alias="thisIsAUnit")

class EnumHasVoidTypeTypes(str, Enum):
    HAS_A_UNIT = "hasAUnit"

class EnumHasVoidTypeHasAUnit(BaseModel):
    type: Literal[EnumHasVoidTypeTypes.HAS_A_UNIT] = EnumHasVoidTypeTypes.HAS_A_UNIT
    content: None

# This enum has a variant associated with unit data
EnumHasVoidType = EnumHasVoidTypeHasAUnit
