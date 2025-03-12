package proto

import (
    "encoding/json"
    "time"
)

type Foo struct {
	ThisIsBits []byte `json:"thisIsBits"`
	ThisIsRedundant []byte `json:"thisIsRedundant"`
}
