from __future__ import annotations

from enum import Enum
from pydantic import BaseModel
from typing import List, Literal, Union


class ItemDetailsFieldValue(BaseModel):
    pass
class AdvancedColorsTypes(str, Enum):
    STRING = "string"
    NUMBER = "number"
    NUMBER_ARRAY = "number-array"
    REALLY_COOL_TYPE = "reallyCoolType"

class AdvancedColorsString(BaseModel):
    type: Literal[AdvancedColorsTypes.STRING] = AdvancedColorsTypes.STRING
    content: str

class AdvancedColorsNumber(BaseModel):
    type: Literal[AdvancedColorsTypes.NUMBER] = AdvancedColorsTypes.NUMBER
    content: int

class AdvancedColorsNumberArray(BaseModel):
    type: Literal[AdvancedColorsTypes.NUMBER_ARRAY] = AdvancedColorsTypes.NUMBER_ARRAY
    content: List[int]

class AdvancedColorsReallyCoolType(BaseModel):
    type: Literal[AdvancedColorsTypes.REALLY_COOL_TYPE] = AdvancedColorsTypes.REALLY_COOL_TYPE
    content: ItemDetailsFieldValue

AdvancedColors = Union[AdvancedColorsString, AdvancedColorsNumber, AdvancedColorsNumberArray, AdvancedColorsReallyCoolType]
