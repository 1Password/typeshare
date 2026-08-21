package proto

import "encoding/json"

type OptionalU16 *uint16

type OptionalU32 *uint32

type FooBar struct {
	Foo OptionalU32 `json:"foo"`
	Bar OptionalU16 `json:"bar"`
}
