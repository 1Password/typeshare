from __future__ import annotations

from enum import Enum
from pydantic import BaseModel, ConfigDict, Field
from typing import Literal, Union


class AlwaysAccept(BaseModel):
    """
    A struct with no target_os. Should be generated when
    we use --target-os.
    """
    pass
class DefinedTwice(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    field_1: str = Field(alias="field1")

class Excluded(BaseModel):
    pass
class ManyStruct(BaseModel):
    pass
class MultipleTargets(BaseModel):
    pass
class NestedNotTarget1(BaseModel):
    pass
class OtherExcluded(BaseModel):
    pass
class AlwaysAcceptEnum(str, Enum):
    VARIANT1 = "Variant1"
    VARIANT2 = "Variant2"
class SomeEnum(str, Enum):
    pass
class TestEnumVariant7Inner(BaseModel):
    """
    Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
    """
    model_config = ConfigDict(populate_by_name=True)

    field_1: str = Field(alias="field1")

class TestEnumVariant9Inner(BaseModel):
    """
    Generated type representing the anonymous struct variant `Variant9` of the `TestEnum` Rust enum
    """
    model_config = ConfigDict(populate_by_name=True)

    field_2: str = Field(alias="field2")

class TestEnumTypes(str, Enum):
    VARIANT_1 = "Variant1"
    VARIANT_5 = "Variant5"
    VARIANT_7 = "Variant7"
    VARIANT_8 = "Variant8"
    VARIANT_9 = "Variant9"

class TestEnumVariant1(BaseModel):
    type: Literal[TestEnumTypes.VARIANT_1] = TestEnumTypes.VARIANT_1

class TestEnumVariant5(BaseModel):
    type: Literal[TestEnumTypes.VARIANT_5] = TestEnumTypes.VARIANT_5

class TestEnumVariant7(BaseModel):
    type: Literal[TestEnumTypes.VARIANT_7] = TestEnumTypes.VARIANT_7
    content: TestEnumVariant7Inner

class TestEnumVariant8(BaseModel):
    type: Literal[TestEnumTypes.VARIANT_8] = TestEnumTypes.VARIANT_8

class TestEnumVariant9(BaseModel):
    type: Literal[TestEnumTypes.VARIANT_9] = TestEnumTypes.VARIANT_9
    content: TestEnumVariant9Inner

TestEnum = Union[TestEnumVariant1, TestEnumVariant5, TestEnumVariant7, TestEnumVariant8, TestEnumVariant9]
