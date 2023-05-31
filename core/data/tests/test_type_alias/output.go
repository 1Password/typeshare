package proto

import (
    "encoding/json"
    "time"
)

type Bar string

type Foo struct {
	Bar Bar `json:"bar"`
}
