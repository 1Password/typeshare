from __future__ import annotations

from enum import Enum
from pydantic import BaseModel
from typing import Dict, List, Literal, Union


class ItemDetailsFieldValue(BaseModel):
    hello: str

class AdvancedColorsTypes(str, Enum):
    STRING = "String"
    NUMBER = "Number"
    NUMBER_ARRAY = "NumberArray"
    REALLY_COOL_TYPE = "ReallyCoolType"
    ARRAY_REALLY_COOL_TYPE = "ArrayReallyCoolType"
    DICTIONARY_REALLY_COOL_TYPE = "DictionaryReallyCoolType"

class AdvancedColorsString(BaseModel):
    t: Literal[AdvancedColorsTypes.STRING] = AdvancedColorsTypes.STRING
    c: str

class AdvancedColorsNumber(BaseModel):
    t: Literal[AdvancedColorsTypes.NUMBER] = AdvancedColorsTypes.NUMBER
    c: int

class AdvancedColorsNumberArray(BaseModel):
    t: Literal[AdvancedColorsTypes.NUMBER_ARRAY] = AdvancedColorsTypes.NUMBER_ARRAY
    c: List[int]

class AdvancedColorsReallyCoolType(BaseModel):
    t: Literal[AdvancedColorsTypes.REALLY_COOL_TYPE] = AdvancedColorsTypes.REALLY_COOL_TYPE
    c: ItemDetailsFieldValue

class AdvancedColorsArrayReallyCoolType(BaseModel):
    t: Literal[AdvancedColorsTypes.ARRAY_REALLY_COOL_TYPE] = AdvancedColorsTypes.ARRAY_REALLY_COOL_TYPE
    c: List[ItemDetailsFieldValue]

class AdvancedColorsDictionaryReallyCoolType(BaseModel):
    t: Literal[AdvancedColorsTypes.DICTIONARY_REALLY_COOL_TYPE] = AdvancedColorsTypes.DICTIONARY_REALLY_COOL_TYPE
    c: Dict[str, ItemDetailsFieldValue]

AdvancedColors = Union[AdvancedColorsString, AdvancedColorsNumber, AdvancedColorsNumberArray, AdvancedColorsReallyCoolType, AdvancedColorsArrayReallyCoolType, AdvancedColorsDictionaryReallyCoolType]
