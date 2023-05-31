package proto

import (
    "encoding/json"
    "time"
)

type Foo struct {
	A int64 `json:"a"`
	B uint64 `json:"b"`
}
