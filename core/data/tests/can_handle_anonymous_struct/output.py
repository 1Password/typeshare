from __future__ import annotations

from enum import Enum
from pydantic import BaseModel
from typing import Literal, Union


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

class AutofilledByTypes(str, Enum):
    US = "Us"
    SOMETHING_ELSE = "SomethingElse"

class AutofilledByUs(BaseModel):
    """
    This field was autofilled by us
    """
    type: Literal[AutofilledByTypes.US] = AutofilledByTypes.US
    content: AutofilledByUsInner

class AutofilledBySomethingElse(BaseModel):
    """
    Something else autofilled this field
    """
    type: Literal[AutofilledByTypes.SOMETHING_ELSE] = AutofilledByTypes.SOMETHING_ELSE
    content: AutofilledBySomethingElseInner

# Enum keeping track of who autofilled a field
AutofilledBy = Union[AutofilledByUs, AutofilledBySomethingElse]
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

class EnumWithManyVariantsTypes(str, Enum):
    UNIT_VARIANT = "UnitVariant"
    TUPLE_VARIANT_STRING = "TupleVariantString"
    ANON_VARIANT = "AnonVariant"
    TUPLE_VARIANT_INT = "TupleVariantInt"
    ANOTHER_UNIT_VARIANT = "AnotherUnitVariant"
    ANOTHER_ANON_VARIANT = "AnotherAnonVariant"

class EnumWithManyVariantsUnitVariant(BaseModel):
    type: Literal[EnumWithManyVariantsTypes.UNIT_VARIANT] = EnumWithManyVariantsTypes.UNIT_VARIANT

class EnumWithManyVariantsTupleVariantString(BaseModel):
    type: Literal[EnumWithManyVariantsTypes.TUPLE_VARIANT_STRING] = EnumWithManyVariantsTypes.TUPLE_VARIANT_STRING
    content: str

class EnumWithManyVariantsAnonVariant(BaseModel):
    type: Literal[EnumWithManyVariantsTypes.ANON_VARIANT] = EnumWithManyVariantsTypes.ANON_VARIANT
    content: EnumWithManyVariantsAnonVariantInner

class EnumWithManyVariantsTupleVariantInt(BaseModel):
    type: Literal[EnumWithManyVariantsTypes.TUPLE_VARIANT_INT] = EnumWithManyVariantsTypes.TUPLE_VARIANT_INT
    content: int

class EnumWithManyVariantsAnotherUnitVariant(BaseModel):
    type: Literal[EnumWithManyVariantsTypes.ANOTHER_UNIT_VARIANT] = EnumWithManyVariantsTypes.ANOTHER_UNIT_VARIANT

class EnumWithManyVariantsAnotherAnonVariant(BaseModel):
    type: Literal[EnumWithManyVariantsTypes.ANOTHER_ANON_VARIANT] = EnumWithManyVariantsTypes.ANOTHER_ANON_VARIANT
    content: EnumWithManyVariantsAnotherAnonVariantInner

# This is a comment (yareek sameek wuz here)
EnumWithManyVariants = Union[EnumWithManyVariantsUnitVariant, EnumWithManyVariantsTupleVariantString, EnumWithManyVariantsAnonVariant, EnumWithManyVariantsTupleVariantInt, EnumWithManyVariantsAnotherUnitVariant, EnumWithManyVariantsAnotherAnonVariant]
