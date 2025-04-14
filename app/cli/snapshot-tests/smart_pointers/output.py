from __future__ import annotations

from enum import Enum
from pydantic import BaseModel
from typing import List, Literal, Union


class ArcyColors(BaseModel):
    """
    This is a comment.
    """
    red: int
    blue: str
    green: List[str]

class CellyColors(BaseModel):
    """
    This is a comment.
    """
    red: str
    blue: List[str]

class CowyColors(BaseModel):
    """
    This is a comment.
    """
    lifetime: str

class LockyColors(BaseModel):
    """
    This is a comment.
    """
    red: str

class MutexyColors(BaseModel):
    """
    This is a comment.
    """
    blue: List[str]
    green: str

class RcyColors(BaseModel):
    """
    This is a comment.
    """
    red: str
    blue: List[str]
    green: str

class BoxyColorsTypes(str, Enum):
    RED = "Red"
    BLUE = "Blue"
    GREEN = "Green"

class BoxyColorsRed(BaseModel):
    type: Literal[BoxyColorsTypes.RED] = BoxyColorsTypes.RED

class BoxyColorsBlue(BaseModel):
    type: Literal[BoxyColorsTypes.BLUE] = BoxyColorsTypes.BLUE

class BoxyColorsGreen(BaseModel):
    type: Literal[BoxyColorsTypes.GREEN] = BoxyColorsTypes.GREEN
    content: str

# This is a comment.
BoxyColors = Union[BoxyColorsRed, BoxyColorsBlue, BoxyColorsGreen]
