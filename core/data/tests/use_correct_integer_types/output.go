package proto

import "encoding/json"

// This is a comment.
type Foo struct {
	A int `json:"a"`
	B int `json:"b"`
	C int `json:"c"`
	E uint8 `json:"e"`
	F uint16 `json:"f"`
	G uint32 `json:"g"`
}
