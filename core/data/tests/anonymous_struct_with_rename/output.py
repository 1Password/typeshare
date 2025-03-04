from __future__ import annotations

from enum import Enum
from pydantic import BaseModel, ConfigDict, Field
from typing import List, Literal, Union


class AnonymousStructWithRenameListInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `List` of the `AnonymousStructWithRename` Rust enum
    """
    list: List[str]

class AnonymousStructWithRenameLongFieldNamesInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `LongFieldNames` of the `AnonymousStructWithRename` Rust enum
    """
    model_config = ConfigDict(populate_by_name=True)

    some_long_field_name: str
    and_: bool = Field(alias="and")
    but_one_more: List[str]

class AnonymousStructWithRenameKebabCaseInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `KebabCase` of the `AnonymousStructWithRename` Rust enum
    """
    model_config = ConfigDict(populate_by_name=True)

    another_list: List[str] = Field(alias="another-list")
    camel_case_string_field: str = Field(alias="camelCaseStringField")
    something_else: bool = Field(alias="something-else")

class AnonymousStructWithRenameTypes(str, Enum):
    LIST = "list"
    LONG_FIELD_NAMES = "longFieldNames"
    KEBAB_CASE = "kebabCase"

class AnonymousStructWithRenameList(BaseModel):
    type: Literal[AnonymousStructWithRenameTypes.LIST] = AnonymousStructWithRenameTypes.LIST
    content: AnonymousStructWithRenameListInner

class AnonymousStructWithRenameLongFieldNames(BaseModel):
    type: Literal[AnonymousStructWithRenameTypes.LONG_FIELD_NAMES] = AnonymousStructWithRenameTypes.LONG_FIELD_NAMES
    content: AnonymousStructWithRenameLongFieldNamesInner

class AnonymousStructWithRenameKebabCase(BaseModel):
    type: Literal[AnonymousStructWithRenameTypes.KEBAB_CASE] = AnonymousStructWithRenameTypes.KEBAB_CASE
    content: AnonymousStructWithRenameKebabCaseInner

AnonymousStructWithRename = Union[AnonymousStructWithRenameList, AnonymousStructWithRenameLongFieldNames, AnonymousStructWithRenameKebabCase]
