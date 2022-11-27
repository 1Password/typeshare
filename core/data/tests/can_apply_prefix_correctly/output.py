"""
 Generated by typeshare 1.0.0
"""
from __future__ import annotations

from pydantic import BaseModel
from typing import List, Dict, Literal


class ItemDetailsFieldValue(BaseModel):
    hello: str


class AdvancedColorsString:
    t: Literal["String"]
    c: str


class AdvancedColorsNumber:
    t: Literal["Number"]
    c: int


class AdvancedColorsNumberArray:
    t: Literal["NumberArray"]
    c: List[int]


class AdvancedColorsReallyCoolType:
    t: Literal["ReallyCoolType"]
    c: ItemDetailsFieldValue


class AdvancedColorsArrayReallyCoolType:
    t: Literal["ArrayReallyCoolType"]
    c: List[ItemDetailsFieldValue]


class AdvancedColorsDictionaryReallyCoolType:
    t: Literal["DictionaryReallyCoolType"]
    c: Dict[str, ItemDetailsFieldValue]


AdvancedColors = AdvancedColorsString | AdvancedColorsNumber | AdvancedColorsNumberArray | AdvancedColorsReallyCoolType | AdvancedColorsArrayReallyCoolType | AdvancedColorsDictionaryReallyCoolType

