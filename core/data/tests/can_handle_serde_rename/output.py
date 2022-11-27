"""
 Generated by typeshare 1.0.0
"""
from __future__ import annotations

from pydantic import Field, BaseModel
from typing import Optional, Annotated, List


class Person(BaseModel):
    """
    This is a comment.
    """
    name: str
    age: int
    extra_special_field_1: Annotated[int, Field(alias="extraSpecialFieldOne")]
    extra_special_field_2: Annotated[Optional[List[str]], Field(alias="extraSpecialFieldTwo")]
    non_standard_data_type: Annotated[OtherType, Field(alias="nonStandardDataType")]
    non_standard_data_type_in_array: Annotated[Optional[List[OtherType]], Field(alias="nonStandardDataTypeInArray")]


class OtherType(BaseModel):
    pass

