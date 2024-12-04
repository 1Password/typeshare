from __future__ import annotations

from pydantic import BaseModel


class A(BaseModel):
    field: int

class AB(BaseModel):
    field: int

class ABC(BaseModel):
    field: int

class OutsideOfModules(BaseModel):
    field: int

