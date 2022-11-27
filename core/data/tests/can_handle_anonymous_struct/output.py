"""
 Generated by typeshare 1.0.0
"""
from __future__ import annotations

from typing import Literal
from pydantic import BaseModel


class AutofilledByUsInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum
    """
    uuid: str
    """
    The UUID for the fill
    """


class AutofilledBySomethingElseInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum
    """
    uuid: str
    """
    The UUID for the fill
    """
    thing: int
    """
    Some other thing
    """






AutofilledBy = AutofilledByUsInner | AutofilledBySomethingElseInner
"""
Enum keeping track of who autofilled a field
"""

class EnumWithManyVariantsAnonVariantInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `AnonVariant` of the `EnumWithManyVariants` Rust enum
    """
    uuid: str


class EnumWithManyVariantsAnotherAnonVariantInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `AnotherAnonVariant` of the `EnumWithManyVariants` Rust enum
    """
    uuid: str
    thing: int


class EnumWithManyVariantsUnitVariant:
    type: Literal["UnitVariant"]


class EnumWithManyVariantsTupleVariantString:
    type: Literal["TupleVariantString"]
    content: str




class EnumWithManyVariantsTupleVariantInt:
    type: Literal["TupleVariantInt"]
    content: int


class EnumWithManyVariantsAnotherUnitVariant:
    type: Literal["AnotherUnitVariant"]




EnumWithManyVariants = EnumWithManyVariantsUnitVariant | EnumWithManyVariantsTupleVariantString | EnumWithManyVariantsAnonVariantInner | EnumWithManyVariantsTupleVariantInt | EnumWithManyVariantsAnotherUnitVariant | EnumWithManyVariantsAnotherAnonVariantInner
"""
This is a comment (yareek sameek wuz here)
"""

