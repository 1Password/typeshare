from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field
from typing import Optional


class A(BaseModel):
    field: int

class B(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    depends_on: A = Field(alias="dependsOn")

class C(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    depends_on: B = Field(alias="dependsOn")

class E(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    depends_on: D = Field(alias="dependsOn")

class D(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    depends_on: C = Field(alias="dependsOn")
    also_depends_on: Optional[E] = Field(alias="alsoDependsOn", default=None)

