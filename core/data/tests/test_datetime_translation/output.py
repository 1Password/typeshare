from __future__ import annotations

from datetime import datetime
from pydantic import BaseModel, BeforeValidator, ConfigDict, Field, PlainSerializer
from typing import Annotated


def serialize_datetime_data(utc_time: datetime) -> str:
        return utc_time.strftime("%Y-%m-%dT%H:%M:%SZ")

def deserialize_datetime_data(utc_time_str: str) -> datetime:
        return datetime.strptime(utc_time_str, "%Y-%m-%dT%H:%M:%SZ")

class Foo(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    time: Annotated[datetime, BeforeValidator(deserialize_datetime_data), PlainSerializer(serialize_datetime_data)]
    time_2: Annotated[datetime, BeforeValidator(deserialize_datetime_data), PlainSerializer(serialize_datetime_data)] = Field(alias="time2")
    time_3: Annotated[datetime, BeforeValidator(deserialize_datetime_data), PlainSerializer(serialize_datetime_data)] = Field(alias="time3")
    non_time: str = Field(alias="nonTime")

