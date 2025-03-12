package proto

import (
    "encoding/json"
    "time"
)

type Foo struct {
	Time time.Time `json:"time"`
	Time2 time.Time `json:"time2"`
}
