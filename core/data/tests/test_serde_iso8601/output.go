package proto

import (
    "encoding/json"
    "time"
)

type Foo struct {
	Time Time `json:"time"`
}
