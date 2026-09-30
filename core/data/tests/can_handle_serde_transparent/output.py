from __future__ import annotations

from pydantic import BaseModel
from typing import List


class Card(BaseModel):
    name: str

Cards = List[Card]

class Player(BaseModel):
    name: str

Players = List[Player]

Pool = Cards

