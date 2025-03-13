package proto

import "encoding/json"

type Foo struct {
	ThisIsBits []byte `json:"thisIsBits"`
	ThisIsRedundant []byte `json:"thisIsRedundant"`
}
